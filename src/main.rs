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
    assigned_ti: Option<usize>,
}
impl Resource {
    fn new(id: usize) -> Self {
        Self {
            id,
            assigned_ti: None,
        }
    }
    fn assign_task(&mut self, task: usize) {
        assert!(self.is_available());
        self.assigned_ti = Some(task);
    }
    fn unassign_task(&mut self) -> usize {
        assert!(!self.is_available());
        let task = self.assigned_ti.unwrap();
        self.assigned_ti = None;
        task
    }
    fn is_available(&self) -> bool {
        self.assigned_ti.is_none()
    }
}

fn main() {
    let mut input_line = String::new();
    stdin().read_line(&mut input_line).unwrap();
    let inputs = input_line.split(' ').collect::<Vec<_>>();
    let n = parse_input!(inputs[0], usize);
    let _k = parse_input!(inputs[1], usize);
    let m = parse_input!(inputs[2], usize);
    let r = parse_input!(inputs[3], usize);

    // Input diffs
    for _ in 0..n {
        let mut input_line = String::new();
        stdin().read_line(&mut input_line).unwrap();
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

    let mut resources = (0..m).map(Resource::new).collect::<Vec<_>>();
    let mut tasks = (0..n).map(Task::new).collect::<Vec<_>>();
    for &(u, v) in &edges {
        tasks[u].nxt_tis.push(v);
        tasks[v].pre_task_cnt += 1;
    }

    // let mut day = 0;
    loop {
        // dbg!(day, &resources, &tasks);
        // Assign tasks
        let ris = resources
            .iter()
            .enumerate()
            .filter(|(_, r)| r.is_available())
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        let tis = tasks
            .iter()
            .enumerate()
            .filter(|(_, t)| t.is_available())
            .map(|(i, _)| i)
            .collect::<Vec<_>>();

        let mut assign_cmd = vec![];
        for (&ri, &ti) in ris.iter().zip(tis.iter()) {
            resources[ri].assign_task(ti);
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
            let completed_ti = resources[ri].unassign_task();
            tasks[completed_ti].unlock();
            let nxt_task_tis = tasks[completed_ti].complete();
            for ti in nxt_task_tis {
                tasks[ti].pre_task_cnt -= 1;
            }
        }
        // day += 1;
    }
}
