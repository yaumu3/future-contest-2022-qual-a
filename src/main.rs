use rand::prelude::*;
use rand_pcg::Mcg128Xsl64;
use std::io::stdin;
const DAY_LIMIT: i32 = 2000;

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
    rng: Mcg128Xsl64,
    id: usize,
    skills: Vec<i32>,
    // ti, start_day
    assigned: Option<(usize, i32)>,
    // ti, elapsed_days
    history: Vec<(usize, i32)>,
}
impl Resource {
    fn new(id: usize, skills_cnt: usize) -> Self {
        let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(id as u64);

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
            rng,
            id,
            skills: s,
            assigned: None,
            history: vec![],
        }
    }
    fn print_skills(&self) {
        let skill_chart = self
            .skills
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        println!("#s {} {}", self.id + 1, skill_chart);
    }
    fn assign_task(&mut self, ti: usize, start_day: i32) {
        assert!(self.is_available());
        self.assigned = Some((ti, start_day));
    }
    fn unassign_task(&mut self, end_day: i32) -> usize {
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
    fn get_est_elapsed_days(&self, diff: &[i32]) -> i32 {
        let skills_cnt = diff.len();
        (0..skills_cnt)
            .map(|k| (diff[k] - self.skills[k]).max(0))
            .sum::<i32>()
            .max(1)
    }
    fn calc_skills_loss_by_history(&self, diffs: &[Vec<i32>]) -> i32 {
        let mut loss = 0;
        for &(ti, elapsed_days) in &self.history {
            let diff = &diffs[ti];
            loss += (elapsed_days - self.get_est_elapsed_days(diff)).abs();
        }
        loss
    }
    fn optimize_skills(&mut self, diffs: &[Vec<i32>], annealer: &mut Annealer) {
        let skills_cnt = self.skills.len();
        let mut cur_loss = self.calc_skills_loss_by_history(diffs);
        let mut best_loss = cur_loss;

        let mut fit_skills = self.skills.clone();
        for _ in 0..1000 {
            let k = self.rng.gen_range(0, skills_cnt);
            let cur_v = self.skills[k];
            let new_v = self.rng.gen_range(0, 20);
            self.skills[k] = new_v;
            let new_loss = self.calc_skills_loss_by_history(diffs);

            if new_loss < best_loss {
                best_loss = new_loss;
                fit_skills = self.skills.clone();
            }

            if annealer.accept((cur_loss - new_loss) as f64) {
                cur_loss = new_loss;
            } else {
                self.skills[k] = cur_v;
            }
        }
        self.skills = fit_skills;
    }
}

struct Annealer {
    t0: f64,
    t1: f64,
    temperture: f64,
    rng: Mcg128Xsl64,
}

impl Annealer {
    fn new(t0: f64, t1: f64) -> Self {
        let seed = 71;
        let rng = rand_pcg::Pcg64Mcg::seed_from_u64(seed);
        Self {
            t0,
            t1,
            temperture: t0,
            rng,
        }
    }

    fn set_temperture(&mut self, tau: f64) {
        self.temperture = self.t0.powf(1.0 - tau) * self.t1.powf(tau);
    }

    fn accept(&mut self, delta: f64) -> bool {
        if delta >= 0. {
            return true;
        }
        let prob = (delta / self.temperture).exp();
        self.rng.gen_bool(prob)
    }
}

fn main() {
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

    let mut resources = (0..m).map(|i| Resource::new(i, k)).collect::<Vec<_>>();
    for res in resources.iter() {
        res.print_skills();
    }

    let mut tasks = (0..n).map(Task::new).collect::<Vec<_>>();
    for &(u, v) in &edges {
        tasks[u].nxt_tis.push(v);
        tasks[v].pre_task_cnt += 1;
    }

    let mut cur_day = 0;
    let mut annealer = Annealer::new(3000.0, 600.0);
    loop {
        let tau = cur_day as f64 / (DAY_LIMIT - 1) as f64;
        annealer.set_temperture(tau);

        // Assign tasks
        let mut ris = resources
            .iter()
            .enumerate()
            .filter(|(_, r)| r.is_available())
            .map(|(i, _)| i)
            .collect::<Vec<_>>();

        let mut tis = tasks
            .iter()
            .enumerate()
            .filter(|(_, t)| t.is_available())
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        tis.sort_by_key(|&ti| tasks[ti].nxt_tis.len());

        let mut assign_cmd = vec![];
        while !ris.is_empty() && !tis.is_empty() {
            let ti = tis.pop().unwrap();
            let ri = {
                if annealer.accept(-1.0) {
                    // Assign resource a task with the shortest estimated days required
                    *ris.iter()
                        .min_by_key(|&&ri| resources[ri].get_est_elapsed_days(&diffs[ti]))
                        .unwrap()
                } else {
                    *ris.iter()
                        .max_by_key(|&&ri| resources[ri].history.len())
                        .unwrap()
                }
            };
            resources[ri].assign_task(ti, cur_day);
            tasks[ti].begin();
            assign_cmd.push(ri + 1);
            assign_cmd.push(ti + 1);
            let idx = ris.iter().position(|&rii| rii == ri).unwrap();
            ris.remove(idx);
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
            resources[ri].optimize_skills(&diffs, &mut annealer);
            resources[ri].print_skills();
        }
        cur_day += 1;
    }
}
