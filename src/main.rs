use rand::prelude::*;
use rand_pcg::Mcg128Xsl64;
use std::cmp::Reverse;
use std::collections::VecDeque;
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
    diff: Vec<i32>,
    diff_norm: i32,
    nxt_tis: Vec<usize>,
    pre_task_cnt: usize,
    is_locked: bool,
    is_done: bool,
}
impl Task {
    fn new(id: usize, diff: &[i32]) -> Self {
        let diff_norm = diff.iter().map(|v| v * v).sum::<i32>();

        Self {
            id,
            diff: diff.to_owned(),
            diff_norm,
            nxt_tis: vec![],
            pre_task_cnt: 0,
            is_locked: false,
            is_done: false,
        }
    }
    fn is_available(&self) -> bool {
        !self.is_locked && !self.is_done && self.pre_task_cnt == 0
    }
    fn is_ready(&self) -> bool {
        self.is_locked && !self.is_done && self.pre_task_cnt == 0
    }
    fn start(&mut self) {
        assert!(self.is_ready());
        self.is_locked = true;
    }
    fn lock(&mut self) {
        assert!(!self.is_locked);
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
    // ti
    queue: VecDeque<usize>,
    // ti, start_day
    working_on: Option<(usize, i32)>,
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
            queue: VecDeque::new(),
            working_on: None,
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
    fn is_busy(&self) -> bool {
        self.working_on.is_some()
    }
    fn is_free(&self) -> bool {
        self.working_on.is_none() && self.queue.is_empty()
    }
    fn queue_task(&mut self, ti: usize) {
        self.queue.push_back(ti);
    }
    fn start_task(&mut self, start_day: i32) -> Option<usize> {
        if self.is_busy() {
            return None;
        }
        match self.queue.pop_front() {
            Some(ti) => {
                self.working_on = Some((ti, start_day));
                Some(ti)
            }
            _ => None,
        }
    }
    fn complete_task(&mut self, end_day: i32) -> usize {
        assert!(self.is_busy());
        let (ti, start_day) = self.working_on.unwrap();
        let elapsed_days = end_day - start_day + 1;
        self.history.push((ti, elapsed_days));
        self.working_on = None;
        ti
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

    let mut tasks = (0..n).map(|i| Task::new(i, &diffs[i])).collect::<Vec<_>>();
    for &(u, v) in &edges {
        tasks[u].nxt_tis.push(v);
        tasks[v].pre_task_cnt += 1;
    }

    let mut ests = vec![vec![0; m]; n];
    for (i, t) in tasks.iter().enumerate() {
        for (j, r) in resources.iter().enumerate() {
            ests[i][j] = r.get_est_elapsed_days(&t.diff);
        }
    }

    let mut cur_day = 0;
    let mut annealer = Annealer::new(3000.0, 600.0);
    let mut rng = rand_pcg::Mcg128Xsl64::seed_from_u64(37);
    loop {
        let tau = cur_day as f64 / (DAY_LIMIT - 1) as f64;
        annealer.set_temperture(tau);

        // Queue tasks
        let mut tis = (0..n)
            .filter(|&ti| tasks[ti].is_available())
            .map(|ti| Some(ti))
            .collect::<Vec<_>>();
        tis.sort_by_key(|&ti| {
            Reverse((
                tasks[ti.unwrap()].nxt_tis.len(),
                tasks[ti.unwrap()].diff_norm,
            ))
        });
        let mut pre_ris = (0..m)
            .filter(|&ri| resources[ri].is_free())
            .map(|ri| Some(ri))
            .collect::<Vec<_>>();
        let mut ris = vec![];
        for &ti in &tis {
            if pre_ris.is_empty() {
                break;
            }
            let ri = *pre_ris
                .iter()
                .min_by_key(|&ri| {
                    resources[ri.unwrap()].get_est_elapsed_days(&tasks[ti.unwrap()].diff)
                })
                .unwrap();
            ris.push(ri);
            let idx = pre_ris.iter().position(|&rii| rii == ri).unwrap();
            pre_ris.remove(idx);
        }
        while !pre_ris.is_empty() {
            ris.push(pre_ris.pop().unwrap());
        }

        let mut best_ris = ris.clone();
        if !tis.is_empty() && !ris.is_empty() {
            while ris.len() < tis.len() {
                ris.push(None);
            }
            while ris.len() > tis.len() {
                tis.push(None);
            }
            assert_eq!(ris.len(), tis.len());

            let mut cur = 0;
            let mut best = 0;
            let mut anl = Annealer::new(200.0, 10.0);

            for i in 0..10000 {
                anl.set_temperture(i as f64 / 10000.0);
                let fm = rng.gen_range(0, tis.len());
                let to = rng.gen_range(0, tis.len());
                if fm == to {
                    continue;
                }
                let delta_sum = {
                    match (tis[fm], tis[to], ris[fm], ris[to]) {
                        (Some(tifm), Some(tito), Some(rifm), None) => {
                            -ests[tifm][rifm] + ests[tito][rifm]
                        }
                        (Some(tifm), Some(tito), None, Some(rito)) => {
                            -ests[tito][rito] + ests[tifm][rito]
                        }
                        (Some(tifm), Some(tito), Some(rifm), Some(rito)) => {
                            -ests[tifm][rifm] - ests[tito][rito]
                                + ests[tifm][rito]
                                + ests[tito][rifm]
                        }
                        (Some(tifm), None, Some(rifm), Some(rito)) => {
                            -ests[tifm][rifm] + ests[tifm][rito]
                        }
                        (None, Some(tito), Some(rifm), Some(rito)) => {
                            -ests[tito][rito] + ests[tito][rifm]
                        }
                        (Some(_), Some(_), None, None) => 0,
                        (None, None, Some(_), Some(_)) => 0,
                        _ => unreachable!(),
                    }
                };
                ris.swap(fm, to);
                cur += delta_sum;
                if cur < best {
                    best_ris = ris.clone();
                    best = cur;
                }
                if !anl.accept(-delta_sum as f64) {
                    ris.swap(fm, to);
                    cur -= delta_sum;
                }
            }
        }

        for (&ti, &ri) in tis.iter().zip(best_ris.iter()) {
            if ti.is_none() || ri.is_none() {
                continue;
            }
            let ri = ri.unwrap();
            let ti = ti.unwrap();
            resources[ri].queue_task(ti);
            tasks[ti].lock();
        }

        // Start tasks
        let mut assign_cmd = vec![];
        for r in &mut resources {
            let ti = r.start_task(cur_day);
            if let Some(ti) = ti {
                tasks[ti].start();
                assign_cmd.push(r.id + 1);
                assign_cmd.push(ti + 1);
            }
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
            let completed_ti = resources[ri].complete_task(cur_day);
            tasks[completed_ti].unlock();
            let nxt_task_tis = tasks[completed_ti].complete();
            for ti in nxt_task_tis {
                tasks[ti].pre_task_cnt -= 1;
            }
            resources[ri].optimize_skills(&diffs, &mut annealer);
            for t in &tasks {
                ests[t.id][ri] = resources[ri].get_est_elapsed_days(&t.diff);
            }
            resources[ri].print_skills();
        }
        cur_day += 1;
    }
}
