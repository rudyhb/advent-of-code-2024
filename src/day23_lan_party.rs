use crate::common::{Context, InputProvider};
use anyhow::Context as AnyhowContext;
use derivative::Derivative;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::str::FromStr;

pub fn run(context: &mut Context) {
    context.add_test_inputs(get_test_inputs());

    let input = context.get_input();
    let input = input.as_str();

    let connections: Vec<Connection> = input
        .lines()
        .map(|line| line.parse())
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    let mut groups_of_three = HashSet::new();
    for (i, connection) in connections.iter().take(connections.len() - 1).enumerate() {
        SetOfThreeCandidate::try_build_set(connection, &connections[i + 1..], &mut groups_of_three);
    }
    let groups_of_three: Vec<SetOfThree> = groups_of_three
        .into_iter()
        .filter_map(|group| group.try_into().ok())
        .collect();
    let groups_with_t = groups_of_three
        .iter()
        .filter(|group| group.any_starts_with_t())
        .map(|group| log::debug!("3: {}", group))
        .count();
    println!("groups that start with t: {}", groups_with_t);

    let parties = Parties {
        connections: ConnectionsIndexed::new(&connections),
    };
    let largest = parties.get_largest_group();

    println!("largest group is size {}", largest.len());
    let largest: Vec<_> = largest.into_iter().map(|v| v.to_string()).collect();
    println!("password: {}", largest.join(","));
}

struct Parties {
    connections: ConnectionsIndexed,
}

fn sorted_vec<T: PartialOrd + Ord + Eq + PartialEq>(items: impl Iterator<Item = T>) -> Vec<T> {
    let mut result: Vec<_> = items.collect();
    result.sort();
    result
}

impl Parties {
    fn get_buddies_between(
        &self,
        left: Computer,
        right: Computer,
    ) -> impl Iterator<Item = Computer> + '_ {
        let left_buddies = self.connections.get_buddies(left);
        let right_buddies = self.connections.get_buddies(right);
        left_buddies
            .iter()
            .copied()
            .filter(|left| right_buddies.contains(left))
            .chain([left, right])
    }
    fn get_largest_group_for(&self, computer: Computer) -> Vec<Computer> {
        let buddies = self.connections.get_buddies(computer);
        let map: HashSet<Vec<Computer>> = buddies
            .iter()
            .map(|&buddy| sorted_vec(self.get_buddies_between(computer, buddy)))
            .collect();
        map.into_iter()
            .filter_map(|buddies| {
                log::trace!(
                    "[{}] testing party size {} found with buddies:\n{:?}",
                    computer,
                    buddies.len(),
                    buddies
                );
                let buddies_ref = &buddies;
                if (0..buddies_ref.len() - 1)
                    .flat_map(|i| {
                        (i + 1..buddies_ref.len()).map(move |j| (buddies_ref[i], buddies_ref[j]))
                    })
                    .all(|(left, right)| self.connections.are_buddies(left, right))
                {
                    log::debug!(
                        "[{}] VALID party size {} found with buddies:\n{:?}",
                        computer,
                        buddies.len(),
                        buddies
                    );
                    Some(buddies)
                } else {
                    None
                }
            })
            .max_by(|left, right| left.len().cmp(&right.len()))
            .unwrap_or_default()
    }
    pub fn get_largest_group(&self) -> Vec<Computer> {
        let mut max = 0;
        let mut largest = vec![];
        for &computer in self.connections.0.keys() {
            let group = self.get_largest_group_for(computer);
            if group.len() > max {
                max = group.len();
                largest = group;
            }
        }
        largest.sort();

        largest
    }
}

struct ConnectionsIndexed(HashMap<Computer, HashSet<Computer>>);

impl ConnectionsIndexed {
    pub fn new(connections: &[Connection]) -> Self {
        let mut result: HashMap<Computer, HashSet<Computer>> = HashMap::new();
        for connection in connections {
            let [left, right] = connection.0;
            result.entry(left).or_default().insert(right);
            result.entry(right).or_default().insert(left);
        }

        Self(result)
    }
    pub fn get_buddies(&self, computer: Computer) -> &HashSet<Computer> {
        self.0.get(&computer).unwrap()
    }
    pub fn are_buddies(&self, left: Computer, right: Computer) -> bool {
        self.0
            .get(&left)
            .map(|buddies| buddies.contains(&right))
            .unwrap_or_default()
    }
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
struct SetOfThree([Computer; 3]);

impl SetOfThree {
    pub fn any_starts_with_t(&self) -> bool {
        self.0.iter().any(|v| v.0.starts_with(&['t']))
    }
}

impl TryFrom<SetOfThreeCandidate> for SetOfThree {
    type Error = ();

    fn try_from(value: SetOfThreeCandidate) -> Result<Self, Self::Error> {
        if value.connections.len() == 3 {
            Ok(Self(value.set))
        } else {
            Err(())
        }
    }
}

#[derive(Clone, Eq, Derivative)]
#[derivative(PartialEq, Hash)]
struct SetOfThreeCandidate {
    set: [Computer; 3],
    #[derivative(PartialEq = "ignore")]
    #[derivative(Hash = "ignore")]
    connections: HashSet<Connection>,
}

impl SetOfThreeCandidate {
    pub fn new(
        left: Computer,
        middle: Computer,
        right: Computer,
        connections: [&Connection; 2],
    ) -> Self {
        let mut array = [left, middle, right];
        array.sort();
        Self {
            set: array,
            connections: connections.into_iter().cloned().collect(),
        }
    }
    pub fn update<'a>(&mut self, connections: impl Iterator<Item = &'a Connection>) {
        self.connections.extend(connections.into_iter().cloned())
    }
    pub fn try_build_set(
        connection: &Connection,
        list: &[Connection],
        aggregator: &mut HashSet<Self>,
    ) {
        for other in list {
            if let Some(trio) = Self::try_build_trio(connection, other) {
                if let Some(mut existing) = aggregator.take(&trio) {
                    existing.update(trio.connections.iter());
                    aggregator.insert(existing);
                } else {
                    aggregator.insert(trio);
                }
            }
        }
    }
    pub fn try_build_trio(left: &Connection, right: &Connection) -> Option<Self> {
        if left.contains(&right.0[0]) || left.contains(&right.0[1]) {
            let set: HashSet<_> = left
                .0
                .iter()
                .copied()
                .chain(right.0.iter().copied())
                .collect();
            assert_eq!(3, set.len(), "inconsistent data");
            let mut set = set.into_iter();
            Some(Self::new(
                set.next().unwrap(),
                set.next().unwrap(),
                set.next().unwrap(),
                [left, right],
            ))
        } else {
            None
        }
    }
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
struct Connection([Computer; 2]);

impl Connection {
    pub fn new(left: Computer, right: Computer) -> Self {
        if left < right {
            Self([left, right])
        } else {
            Self([right, left])
        }
    }
    pub fn contains(&self, computer: &Computer) -> bool {
        self.0.contains(computer)
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
struct Computer([char; 2]);

impl Computer {
    pub fn new(name: &str) -> anyhow::Result<Self> {
        if name.len() == 2 {
            Ok(Self(name.chars().collect::<Vec<_>>().try_into().unwrap()))
        } else {
            Err(anyhow::anyhow!("invalid computer string length"))
        }
    }
}

impl FromStr for Connection {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('-');
        let mut get_next = || -> anyhow::Result<Computer> {
            let next = parts.next().context("no dash in connection string")?;
            Computer::new(next)
        };
        Ok(Self::new(get_next()?, get_next()?))
    }
}

impl Display for Computer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.0[0], self.0[1])
    }
}

impl Display for Connection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.0[0], self.0[1])
    }
}

impl Display for SetOfThree {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{},{}", self.0[0], self.0[1], self.0[2])
    }
}

impl Debug for Computer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

fn get_test_inputs() -> impl Iterator<Item = Box<InputProvider>> {
    ["kh-tc
qp-kh
de-cg
ka-co
yn-aq
qp-ub
cg-tb
vc-aq
tb-ka
wh-tc
yn-cg
kh-ub
ta-co
de-co
tc-td
tb-wq
wh-td
ta-ka
td-qp
aq-cg
wq-ub
ub-vc
de-ta
wq-aq
wq-vc
wh-yn
ka-de
kh-ta
co-tc
wh-qp
tb-vc
td-yn"]
    .into_iter()
    .map(|input| Box::new(move || input.into()) as Box<InputProvider>)
}
