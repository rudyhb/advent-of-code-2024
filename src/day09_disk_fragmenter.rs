use crate::common::{Context, InputProvider};
use anyhow::Context as AnyhowContext;
use std::collections::VecDeque;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub fn run(context: &mut Context) {
    context.add_test_inputs(get_test_inputs());

    let input = context.get_input();
    let input = input.as_str();

    let checksum = solve(input, false);
    println!("solution 1: {}", checksum);
    let checksum = solve(input, true);
    println!("solution 2: {}", checksum);
}

fn solve(input: &str, is_v2: bool) -> usize {
    let mut disk: Disk = input.parse().unwrap();
    if is_v2 {
        disk.make_v2();
    }
    log::debug!("{}", disk);
    while disk.defragment_next() {
        log::debug!("{}", disk);
    }
    disk.get_checksum()
}

struct Disk {
    files: VecDeque<Blocks>,
    last: usize,
    is_v2: bool,
    last_id: usize,
}

impl Display for Disk {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\n")?;
        for file in self.files.iter() {
            match file {
                Blocks::FreeSpace(space) => {
                    write!(
                        f,
                        "{}",
                        std::iter::repeat('.').take(*space).collect::<String>()
                    )?;
                }
                Blocks::File(file) => {
                    write!(
                        f,
                        "{}",
                        std::iter::repeat(file.id.to_string())
                            .take(file.count)
                            .collect::<String>()
                    )?;
                }
            }
        }

        Ok(())
    }
}

impl Disk {
    fn new(files: VecDeque<Blocks>) -> Self {
        Self {
            files,
            last: 0,
            is_v2: false,
            last_id: usize::MAX,
        }
    }
    fn make_v2(&mut self) {
        self.is_v2 = true;
        self.last = self.files.len() - 1;
    }
    fn get_checksum(&self) -> usize {
        let mut i = 0;
        self.files
            .iter()
            .filter_map(|file| match file {
                Blocks::FreeSpace(space) => {
                    i += *space;
                    None
                }
                Blocks::File(file) => {
                    let checksum = file.get_checksum(i);
                    i += file.count;
                    Some(checksum)
                }
            })
            .sum()
    }
    pub fn defragment_next(&mut self) -> bool {
        if self.is_v2 {
            self.defragment_next_v2()
        } else {
            self.defragment_next_v1()
        }
    }
    pub fn defragment_next_v1(&mut self) -> bool {
        let (i, space) = if let Some((i, space)) = (self.last..self.files.len())
            .into_iter()
            .filter_map(|i| self.files[i].get_free_space().map(|s| (i, s)))
            .next()
        {
            (i, space)
        } else {
            return false;
        };

        self.last = i;
        loop {
            let last_file = self.files.back_mut().context("no last file").unwrap();
            match last_file {
                Blocks::FreeSpace(_) => {
                    self.files.pop_back();
                }
                Blocks::File(file) => {
                    if file.count < space {
                        self.files[i] = Blocks::FreeSpace(space - file.count);
                        let file = self.files.pop_back().unwrap();
                        self.files.insert(i, file);
                    } else if file.count == space {
                        let file = self.files.pop_back().unwrap();
                        self.files[i] = file;
                    } else {
                        file.count -= space;
                        let id = file.id;
                        self.files[i] = Blocks::File(File::new(id, space));
                    }
                    break true;
                }
            }
        }
    }
    pub fn defragment_next_v2(&mut self) -> bool {
        let next_file = loop {
            if self.last == 0 {
                return false;
            }
            match &self.files[self.last] {
                Blocks::FreeSpace(_) => {}
                Blocks::File(file) => {
                    if file.id < self.last_id {
                        break file;
                    }
                }
            }
            self.last = self.last.saturating_sub(1);
        };
        log::debug!("working with file {}", next_file.id);
        let (i, space) = if let Some((i, space)) = self
            .files
            .iter()
            .take(self.last)
            .enumerate()
            .filter_map(|(i, file)| {
                file.get_free_space().and_then(|s| {
                    if s >= next_file.count {
                        Some((i, s))
                    } else {
                        None
                    }
                })
            })
            .next()
        {
            (i, space)
        } else {
            self.last = self.last.saturating_sub(1);
            return true;
        };

        let next_file_count = next_file.count;
        self.last_id = next_file.id;

        let block = std::mem::replace(
            &mut self.files[self.last],
            Blocks::FreeSpace(next_file_count),
        );

        if next_file_count < space {
            self.files[i] = Blocks::FreeSpace(space - next_file_count);
            self.files.insert(i, block);
            self.last += 1;
        } else if next_file_count == space {
            self.files[i] = block;
        } else {
            unreachable!()
        }

        self.merge_spaces_around_last();

        self.last = self.last.saturating_sub(1);
        true
    }
    fn merge_spaces_around_last(&mut self) {
        assert!(self.last > 0);

        let mut space = self.files[self.last]
            .get_free_space()
            .expect("merge spaces should reference a FreeSpace block");

        if let Some(left) = self.files[self.last - 1].get_free_space() {
            space += left;
            self.files.remove(self.last - 1);
            self.last -= 1;
        }
        if self.last <= self.files.len() - 2 {
            if let Some(right) = self.files[self.last + 1].get_free_space() {
                space += right;
                self.files.remove(self.last + 1);
            }
        }

        self.files[self.last] = Blocks::FreeSpace(space);
    }
}

#[derive(Debug)]
enum Blocks {
    FreeSpace(usize),
    File(File),
}

impl Blocks {
    pub fn get_free_space(&self) -> Option<usize> {
        if let Self::FreeSpace(space) = self {
            Some(*space)
        } else {
            None
        }
    }
}

#[derive(Debug)]
struct File {
    id: usize,
    count: usize,
}

impl File {
    fn new(id: usize, count: usize) -> Self {
        Self { id, count }
    }
    pub fn get_checksum(&self, start: usize) -> usize {
        (start..start + self.count).map(|pos| self.id * pos).sum()
    }
}

impl FromStr for Disk {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut is_free_space = true;
        let mut id = 0;
        Ok(Self::new(
            s.chars()
                .into_iter()
                .map(|c| {
                    let space: u32 = c.to_digit(10).context("cannot parse space number")?;
                    is_free_space = !is_free_space;
                    if is_free_space {
                        Ok::<Blocks, anyhow::Error>(Blocks::FreeSpace(space as usize))
                    } else {
                        let this_id = id;
                        id += 1;
                        Ok(Blocks::File(File::new(this_id, space as usize)))
                    }
                })
                .collect::<Result<VecDeque<_>, _>>()?,
        ))
    }
}

fn get_test_inputs() -> impl Iterator<Item = Box<InputProvider>> {
    ["2333133121414131402"]
        .into_iter()
        .map(|input| Box::new(move || input.into()) as Box<InputProvider>)
}
