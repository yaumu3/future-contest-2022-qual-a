use rand::prelude::*;
use rand_pcg::Mcg128Xsl64;
use std::io::stdin;

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

#[derive(Debug, Clone)]
struct Task {
    id: usize,
    nxt_tis: Vec<usize>,
    pre_task_cnt: usize,
    is_locked: bool,
    is_done: bool,
}
impl Task {
    fn new(id: usize) -> Self {
        Self {
            id,
            nxt_tis: vec![],
            pre_task_cnt: 0,
            is_locked: false,
            is_done: false,
        }
    }
    fn is_available(&self) -> bool {
        !self.is_locked && !self.is_done && self.pre_task_cnt == 0
    }
    fn begin(&mut self) {
        assert!(self.is_available());
        self.is_locked = true;
    }
    fn unlock(&mut self) {
        assert!(self.is_locked);
        self.is_locked = false;
    }
    fn complete(&mut self) -> Vec<usize> {
        assert!(!self.is_locked);
        self.is_locked = false;
        self.is_done = true;
        self.nxt_tis.clone()
    }
}

#[derive(Debug, Clone)]
struct Resource {
    id: usize,
    skills: Vec<i32>,
    // ti, start_day
    assigned: Option<(usize, usize)>,
    // ti, elapsed_days
    history: Vec<(usize, usize)>,
}
impl Resource {
    fn new(id: usize, skills_cnt: usize, rng: &mut Mcg128Xsl64) -> Self {
        // Initialize `skills` with random
        let mut b = vec![0.0; skills_cnt];
        for b in b.iter_mut() {
            *b = f64::abs(rng.sample(rand_distr::StandardNormal));
        }
        let mul = rng.gen_range(20.0, 60.0) / b.iter().map(|x| x * x).sum::<f64>().sqrt();
        let mut s = vec![0; skills_cnt];
        for i in 0..skills_cnt {
            s[i] = (b[i] * mul).round() as i32;
        }

        Self {
            id,
            skills: s,
            assigned: None,
            history: vec![],
        }
    }
    fn assign_task(&mut self, ti: usize, start_day: usize) {
        assert!(self.is_available());
        self.assigned = Some((ti, start_day));
    }
    fn unassign_task(&mut self, end_day: usize) -> usize {
        assert!(!self.is_available());
        let (ti, start_day) = self.assigned.unwrap();
        let elapsed_days = end_day - start_day + 1;
        self.history.push((ti, elapsed_days));
        self.assigned = None;
        ti
    }
    fn is_available(&self) -> bool {
        self.assigned.is_none()
    }
}

fn main() {
    let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(71);

    let mut input_line = String::new();
    stdin().read_line(&mut input_line).unwrap();
    let inputs = input_line.split(' ').collect::<Vec<_>>();
    let n = parse_input!(inputs[0], usize);
    let m = parse_input!(inputs[1], usize);
    let k = parse_input!(inputs[2], usize);
    let r = parse_input!(inputs[3], usize);

    // Input diffs
    let mut diffs = vec![vec![0; k]; n];
    for diff in diffs.iter_mut() {
        let mut input_line = String::new();
        stdin().read_line(&mut input_line).unwrap();
        let inputs = input_line.split(' ').collect::<Vec<_>>();
        for (j, v) in inputs.iter().enumerate() {
            diff[j] = parse_input!(v, i32);
        }
    }

    // Input edges
    let mut edges = vec![];
    for _ in 0..r {
        let mut input_line = String::new();
        stdin().read_line(&mut input_line).unwrap();
        let inputs = input_line.split(' ').collect::<Vec<_>>();
        let edge = (
            parse_input!(inputs[0], usize) - 1,
            parse_input!(inputs[1], usize) - 1,
        );
        edges.push(edge);
    }

    let mut resources = (0..m)
        .map(|i| Resource::new(i, k, &mut rng))
        .collect::<Vec<_>>();

    let mut tasks = (0..n).map(Task::new).collect::<Vec<_>>();
    for &(u, v) in &edges {
        tasks[u].nxt_tis.push(v);
        tasks[v].pre_task_cnt += 1;
    }

    let mut cur_day = 0;
    loop {
        // Output estimated skill
        for (i, res) in resources.iter().enumerate() {
            let skill_chart = res
                .skills
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            println!("#s {} {}", i + 1, skill_chart);
        }

        // dbg!(day, &resources, &tasks);
        // Assign tasks
        let mut ris = resources
            .iter()
            .enumerate()
            .filter(|(_, r)| r.is_available())
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        ris.sort_by_key(|&ri| n - resources[ri].history.len());

        let mut tis = tasks
            .iter()
            .enumerate()
            .filter(|(_, t)| t.is_available())
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        tis.sort_by_key(|&ti| n - tasks[ti].nxt_tis.len());

        let mut assign_cmd = vec![];
        for (&ri, &ti) in ris.iter().zip(tis.iter()) {
            resources[ri].assign_task(ti, cur_day);
            tasks[ti].begin();
            assign_cmd.push(ri + 1);
            assign_cmd.push(ti + 1);
        }
        let assign_len = assign_cmd.len() / 2;
        let assign_cmd = assign_cmd
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        println!("{} {}", assign_len, assign_cmd);

        // Check completed tasks
        let mut input_line = String::new();
        stdin().read_line(&mut input_line).unwrap();
        let freed_resources = input_line
            .split(' ')
            .map(|v| parse_input!(v, isize))
            .collect::<Vec<_>>();
        // dbg!(&freed_resources);

        if freed_resources[0] == -1 {
            break;
        }
        for &ri in &freed_resources[1..] {
            let ri = ri as usize - 1;
            let completed_ti = resources[ri].unassign_task(cur_day);
            tasks[completed_ti].unlock();
            let nxt_task_tis = tasks[completed_ti].complete();
            for ti in nxt_task_tis {
                tasks[ti].pre_task_cnt -= 1;
            }
        }
        cur_day += 1;
    }
}
