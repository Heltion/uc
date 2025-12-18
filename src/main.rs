use std::io::Write;

fn read_many_usize() -> Vec<usize> {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input
        .trim()
        .split_whitespace()
        .map(|s| s.parse().unwrap())
        .collect()
}
fn flush() {
    std::io::stdout().flush().unwrap();
}
#[derive(Clone, Default)]
struct CoreConfig {
    computation_paths: Vec<Vec<usize>>,
    computation_costs: Vec<Vec<usize>>,
    special_costs: Vec<usize>,
    number_of_cores: usize,
    switch_cost: usize,
    receive_cost: usize,
}
impl CoreConfig {
    fn initialize() -> Self {
        let mut computation_paths: Vec<_> = (0..7)
            .map(|_| {
                let mut path = read_many_usize();
                path.remove(0);
                path
            })
            .collect();
        computation_paths.insert(0, Vec::new());
        let mut computation_costs: Vec<_> = (0..20).map(|_| read_many_usize()).collect();
        computation_costs.insert(0, Vec::new());
        let special_costs = read_many_usize();
        let input = read_many_usize();
        Self {
            computation_paths,
            computation_costs,
            special_costs,
            number_of_cores: input[0],
            switch_cost: input[1],
            receive_cost: input[2],
        }
    }
}
#[derive(Clone, Copy, Default)]
struct Packet {
    id: usize,
    arrive: usize,
    type_id: usize,
    timeout: usize,
    current_node_index: usize,
    current_core_id: usize,
    work: Option<usize>,
}
#[derive(Default)]
struct Core {
    config: CoreConfig,
    last_time: Vec<usize>,
}
impl Core {
    fn new(config: CoreConfig) -> Self {
        let number_of_cores = config.number_of_cores;
        Self {
            config,
            last_time: vec![1; number_of_cores + 1],
        }
    }
    fn receive(&mut self, t: usize) -> Vec<Packet> {
        println!("R {}", t);
        flush();
        let p = read_many_usize()[0];
        (0..p)
            .map(|_| {
                let input = read_many_usize();
                Packet {
                    id: input[0],
                    arrive: input[1],
                    type_id: input[2],
                    timeout: input[3],
                    ..Default::default()
                }
            })
            .collect()
    }
    fn execute(
        &mut self,
        t: usize,
        core_id: usize,
        node_id: usize,
        packets: &mut [Packet],
    ) -> usize {
        if node_id == 8 {
            for packet in packets.iter_mut() {
                if packet.work.is_none() {
                    self.query(t, packet);
                }
            }
        }
        print!("E {} {} {} {}", t, core_id, node_id, packets.len());
        let mut total_cost = self.config.computation_costs[node_id][packets.len()];
        for packet in packets {
            assert_eq!(
                self.config.computation_paths[packet.type_id][packet.current_node_index],
                node_id
            );
            packet.current_node_index += 1;
            if packet.current_core_id != core_id {
                packet.current_core_id = core_id;
                total_cost += self.config.switch_cost;
            }
            if node_id == 8 {
                let work = packet.work.unwrap();
                for i in 0..8 {
                    if (work >> i) & 1 == 1 {
                        total_cost += self.config.special_costs[i];
                    }
                }
            }
            print!(" {}", packet.id);
        }
        println!();
        flush();
        total_cost
    }
    fn query(&mut self, t: usize, packet: &mut Packet) {
        println!("Q {} {}", t, packet.id);
        flush();
        let work = read_many_usize()[0];
        packet.work = Some(work);
    }
    fn finish(&mut self) {
        println!("F");
        flush();
    }
    fn get_current_node_id(&self, packet: &Packet) -> usize {
        self.config.computation_paths[packet.type_id][packet.current_node_index]
    }
    fn interactive(&mut self) -> bool {
        let n = read_many_usize()[0];
        let mut number_of_received_packets = 0;
        let mut queue: std::collections::HashMap<usize, Vec<Packet>> =
            std::collections::HashMap::new();
        let mut active: Vec<Packet> = Vec::new();
        for t in 1.. {
            if number_of_received_packets == n && active.is_empty() && queue.is_empty() {
                self.finish();
                break;
            }
            if t >= self.last_time[0] && number_of_received_packets < n {
                let packets = self.receive(t);
                number_of_received_packets += packets.len();
                let done = t + self.config.receive_cost;
                queue.entry(done).or_default().extend(packets);
                self.last_time[0] = done;
            }
            if let Some(packets) = queue.remove(&t) {
                active.extend(packets);
            }
            for core in 1..=self.config.number_of_cores {
                if self.last_time[core] > t || active.is_empty() {
                    continue;
                }
                active.sort_by_key(|p| {
                    (
                        p.current_core_id == core,
                        std::cmp::Reverse(p.arrive + p.timeout),
                    )
                });
                let mut to_execute: Vec<Packet> = active.split_off(active.len() - 1);
                let node_id = self.get_current_node_id(&to_execute[0]);
                for i in (0..active.len()).rev() {
                    if to_execute.len() == self.config.computation_costs[node_id].len() - 1 {
                        break;
                    }
                    if self.get_current_node_id(&active[i]) == node_id {
                        to_execute.push(active.swap_remove(i));
                    }
                }
                let cost = self.execute(t, core, node_id, &mut to_execute);
                let done = t + cost;
                to_execute.retain(|p| {
                    p.current_node_index < self.config.computation_paths[p.type_id].len()
                });
                queue.entry(done).or_default().extend(to_execute);
                self.last_time[core] = done;
            }
        }
        true
    }
}
fn main() {
    let core_config = CoreConfig::initialize();
    for _ in 0..5 {
        let mut core = Core::new(core_config.clone());
        if !core.interactive() {
            break;
        }
    }
}
