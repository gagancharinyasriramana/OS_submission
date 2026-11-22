// Workload-Aware CPU Scheduling Research Lab
// Implements discrete-event scheduling simulation for FCFS, SJF, Round Robin, Priority, and Workload-Aware Selector
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkloadClass {
    CpuBound,
    IoBound,
    Interactive,
    RealTimeInspired,
    Mixed,
}

impl WorkloadClass {
    pub fn name(&self) -> &'static str {
        match self {
            WorkloadClass::CpuBound => "CPU-Bound",
            WorkloadClass::IoBound => "I/O-Bound",
            WorkloadClass::Interactive => "Interactive",
            WorkloadClass::RealTimeInspired => "Real-Time-Inspired",
            WorkloadClass::Mixed => "Mixed Workload",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Algorithm {
    Fcfs,
    Sjf,
    RoundRobin,
    Priority,
    WorkloadAware,
}

impl Algorithm {
    pub fn name(&self) -> &'static str {
        match self {
            Algorithm::Fcfs => "FCFS",
            Algorithm::Sjf => "SJF (Non-Preemptive)",
            Algorithm::RoundRobin => "Round Robin (RR)",
            Algorithm::Priority => "Priority (with Aging)",
            Algorithm::WorkloadAware => "Workload-Aware Selector",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoFreq {
    None,
    Occasional,
    Frequent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    New,
    Ready,
    Running,
    Blocked,
    Completed,
}

#[derive(Debug, Clone)]
pub struct Task {
    pub id: usize,
    pub arrival_time: u32,
    pub cpu_burst: u32,
    pub remaining_burst: u32,
    pub priority: u32, // 1 = highest, 10 = lowest
    pub io_freq: IoFreq,
    pub deadline: Option<u32>,
    pub first_start_time: Option<u32>,
    pub completion_time: Option<u32>,
    pub total_waiting_time: u32,
    pub response_time: u32,
    pub turnaround_time: u32,
    pub state: TaskState,
    pub io_remaining: u32,
}

impl Task {
    pub fn new(id: usize, arrival: u32, burst: u32, priority: u32, io_freq: IoFreq, deadline: Option<u32>) -> Self {
        Task {
            id,
            arrival_time: arrival,
            cpu_burst: burst,
            remaining_burst: burst,
            priority,
            io_freq,
            deadline,
            first_start_time: None,
            completion_time: None,
            total_waiting_time: 0,
            response_time: 0,
            turnaround_time: 0,
            state: TaskState::New,
            io_remaining: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WorkloadTrace {
    pub class: WorkloadClass,
    pub tasks: Vec<Task>,
    pub seed: u32,
}

#[derive(Debug, Clone)]
pub struct SchedulingResult {
    pub algorithm: Algorithm,
    pub workload_class: WorkloadClass,
    pub task_count: usize,
    pub avg_response_time: f32,
    pub avg_waiting_time: f32,
    pub avg_turnaround_time: f32,
    pub throughput: f32, // completed tasks per 100 time units
    pub context_switches: u32,
    pub deadline_misses: u32,
    pub total_time: u32,
    pub completed_tasks: usize,
    pub selection_reason: String,
    pub chosen_algorithm: Algorithm,
}

// Classification characteristics
#[derive(Debug, Clone)]
pub struct WorkloadCharacteristics {
    pub task_count: usize,
    pub avg_burst: f32,
    pub short_burst_ratio: f32,
    pub long_burst_ratio: f32,
    pub io_freq: IoFreq,
    pub has_deadlines: bool,
    pub staggered_arrivals: bool,
    pub classified_as: WorkloadClass,
    pub selected_algo: Algorithm,
    pub reason: String,
}

pub fn analyze_workload(tasks: &[Task]) -> WorkloadCharacteristics {
    let task_count = tasks.len();
    if task_count == 0 {
        return WorkloadCharacteristics {
            task_count: 0,
            avg_burst: 0.0,
            short_burst_ratio: 0.0,
            long_burst_ratio: 0.0,
            io_freq: IoFreq::None,
            has_deadlines: false,
            staggered_arrivals: false,
            classified_as: WorkloadClass::Mixed,
            selected_algo: Algorithm::RoundRobin,
            reason: String::from("Empty workload, default fallback to Round Robin"),
        };
    }

    let mut total_burst = 0;
    let mut short_count = 0;
    let mut long_count = 0;
    let mut has_deadlines = false;
    let mut io_occasional = 0;
    let mut io_frequent = 0;
    let first_arrival = tasks[0].arrival_time;
    let mut staggered = false;

    for t in tasks {
        total_burst += t.cpu_burst;
        if t.cpu_burst <= 5 {
            short_count += 1;
        } else if t.cpu_burst >= 21 {
            long_count += 1;
        }
        if t.deadline.is_some() {
            has_deadlines = true;
        }
        if t.arrival_time != first_arrival {
            staggered = true;
        }
        match t.io_freq {
            IoFreq::Occasional => io_occasional += 1,
            IoFreq::Frequent => io_frequent += 1,
            IoFreq::None => {}
        }
    }

    let avg_burst = total_burst as f32 / task_count as f32;
    let short_ratio = short_count as f32 / task_count as f32;
    let long_ratio = long_count as f32 / task_count as f32;

    let dominant_io = if io_frequent >= task_count / 2 {
        IoFreq::Frequent
    } else if io_occasional >= task_count / 2 {
        IoFreq::Occasional
    } else {
        IoFreq::None
    };

    // Rule-based classification & selection (Section 9.7 C)
    let (classified, algo, reason) = if has_deadlines {
        (
            WorkloadClass::RealTimeInspired,
            Algorithm::Priority,
            String::from("Deadlines detected: Priority scheduling prioritized by urgency"),
        )
    } else if dominant_io == IoFreq::Frequent && short_ratio >= 0.5 {
        (
            WorkloadClass::IoBound,
            Algorithm::RoundRobin,
            String::from("Frequent I/O blocking with short bursts: Round Robin maximizes responsiveness"),
        )
    } else if short_ratio >= 0.6 && staggered {
        (
            WorkloadClass::Interactive,
            Algorithm::RoundRobin,
            String::from("Short tasks with staggered arrivals: Round Robin prevents head-of-line blocking"),
        )
    } else if avg_burst >= 20.0 && dominant_io == IoFreq::None {
        (
            WorkloadClass::CpuBound,
            Algorithm::Fcfs,
            String::from("Long CPU bursts with minimal I/O: FCFS minimizes context-switch overhead"),
        )
    } else {
        (
            WorkloadClass::Mixed,
            Algorithm::RoundRobin,
            String::from("Heterogeneous task mix: Round Robin used as balanced fallback strategy"),
        )
    };

    WorkloadCharacteristics {
        task_count,
        avg_burst,
        short_burst_ratio: short_ratio,
        long_burst_ratio: long_ratio,
        io_freq: dominant_io,
        has_deadlines,
        staggered_arrivals: staggered,
        classified_as: classified,
        selected_algo: algo,
        reason,
    }
}

// Generate reproducible workload traces (Section 9.4)
pub fn generate_workload(class: WorkloadClass, task_count: usize, seed: u32) -> WorkloadTrace {
    let mut tasks = Vec::with_capacity(task_count);
    let mut state = if seed == 0 { 0x5EED1234 } else { seed };

    let mut next_u32 = || -> u32 {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        state
    };

    let mut range = |min: u32, max: u32| -> u32 {
        if min >= max { min } else { min + (next_u32() % (max - min + 1)) }
    };

    match class {
        WorkloadClass::CpuBound => {
            // Long bursts (25-80), no I/O, simultaneous or slight arrival
            for i in 1..=task_count {
                let burst = range(25, 80);
                let arrival = range(0, 5);
                let priority = range(1, 5);
                tasks.push(Task::new(i, arrival, burst, priority, IoFreq::None, None));
            }
        }
        WorkloadClass::IoBound => {
            // Short bursts (1-5), frequent I/O blocking
            for i in 1..=task_count {
                let burst = range(2, 6);
                let arrival = range(0, 10);
                let priority = range(1, 5);
                tasks.push(Task::new(i, arrival, burst, priority, IoFreq::Frequent, None));
            }
        }
        WorkloadClass::Interactive => {
            // Staggered short bursts (1-5) mixed with occasional medium burst (8-12)
            for i in 1..=task_count {
                let burst = if i % 4 == 0 { range(8, 14) } else { range(1, 4) };
                let arrival = (i as u32 - 1) * 3; // Staggered intervals
                let priority = range(1, 5);
                tasks.push(Task::new(i, arrival, burst, priority, IoFreq::Occasional, None));
            }
        }
        WorkloadClass::RealTimeInspired => {
            // Deadlines present, medium bursts (5-15)
            for i in 1..=task_count {
                let burst = range(4, 12);
                let arrival = (i as u32 - 1) * 4;
                let deadline = arrival + burst + range(10, 25);
                let priority = range(1, 5);
                tasks.push(Task::new(i, arrival, burst, priority, IoFreq::None, Some(deadline)));
            }
        }
        WorkloadClass::Mixed => {
            // Mixture of CPU-bound, I/O-bound, and interactive tasks
            for i in 1..=task_count {
                let (burst, io, arrival) = match i % 3 {
                    0 => (range(25, 60), IoFreq::None, range(0, 5)),        // CPU bound
                    1 => (range(2, 5), IoFreq::Frequent, range(0, 15)),      // I/O bound
                    _ => (range(3, 8), IoFreq::Occasional, (i as u32) * 2), // Interactive
                };
                let priority = range(1, 5);
                tasks.push(Task::new(i, arrival, burst, priority, io, None));
            }
        }
    }

    WorkloadTrace { class, tasks, seed }
}

// Discrete-event CPU scheduling simulator
pub fn simulate(algo: Algorithm, trace: &WorkloadTrace, quantum: u32) -> SchedulingResult {
    let mut tasks = trace.tasks.clone();
    let task_count = tasks.len();
    if task_count == 0 {
        return SchedulingResult {
            algorithm: algo,
            workload_class: trace.class,
            task_count: 0,
            avg_response_time: 0.0,
            avg_waiting_time: 0.0,
            avg_turnaround_time: 0.0,
            throughput: 0.0,
            context_switches: 0,
            deadline_misses: 0,
            total_time: 0,
            completed_tasks: 0,
            selection_reason: String::new(),
            chosen_algorithm: algo,
        };
    }

    // If WorkloadAware, run classification to choose actual algorithm
    let (effective_algo, reason) = if algo == Algorithm::WorkloadAware {
        let charac = analyze_workload(&tasks);
        (charac.selected_algo, charac.reason)
    } else {
        (algo, String::from("Explicit algorithm execution"))
    };

    let mut current_time: u32 = 0;
    let mut completed_count = 0;
    let mut context_switches: u32 = 0;
    let mut last_running_id: Option<usize> = None;
    let mut ready_queue: Vec<usize> = Vec::new(); // indices into tasks
    let mut quantum_left: u32 = quantum;

    // Simulation loop
    while completed_count < task_count {
        // 1. Check newly arriving tasks
        for (i, t) in tasks.iter_mut().enumerate() {
            if t.state == TaskState::New && t.arrival_time <= current_time {
                t.state = TaskState::Ready;
                ready_queue.push(i);
            }
        }

        // 2. Advance I/O for blocked tasks
        for (i, t) in tasks.iter_mut().enumerate() {
            if t.state == TaskState::Blocked {
                if t.io_remaining > 0 {
                    t.io_remaining -= 1;
                }
                if t.io_remaining == 0 {
                    t.state = TaskState::Ready;
                    ready_queue.push(i);
                }
            }
        }

        // 3. Select next task to run
        if ready_queue.is_empty() {
            // CPU is idle, advance time
            current_time += 1;
            continue;
        }

        let selected_idx = match effective_algo {
            Algorithm::Fcfs => {
                // Select task with earliest arrival in ready queue
                let mut best_q_idx = 0;
                let mut best_arrival = tasks[ready_queue[0]].arrival_time;
                for (q_idx, &t_idx) in ready_queue.iter().enumerate() {
                    if tasks[t_idx].arrival_time < best_arrival {
                        best_arrival = tasks[t_idx].arrival_time;
                        best_q_idx = q_idx;
                    }
                }
                ready_queue.remove(best_q_idx)
            }
            Algorithm::Sjf => {
                // Shortest Job First: minimum remaining burst (non-preemptive)
                let mut best_q_idx = 0;
                let mut min_burst = tasks[ready_queue[0]].remaining_burst;
                for (q_idx, t_idx) in ready_query_non_preemptive(&ready_queue, &tasks) {
                    if tasks[t_idx].remaining_burst < min_burst {
                        min_burst = tasks[t_idx].remaining_burst;
                        best_q_idx = q_idx;
                    }
                }
                ready_queue.remove(best_q_idx)
            }
            Algorithm::RoundRobin => {
                // Pop first from queue
                ready_queue.remove(0)
            }
            Algorithm::Priority => {
                // Priority scheduling (1 = highest priority). Aging applied: reduce priority number if waited long
                let mut best_q_idx = 0;
                let mut best_prio = effective_priority(&tasks[ready_queue[0]], current_time);
                for (q_idx, &t_idx) in ready_queue.iter().enumerate() {
                    let p = effective_priority(&tasks[t_idx], current_time);
                    if p < best_prio {
                        best_prio = p;
                        best_q_idx = q_idx;
                    }
                }
                ready_queue.remove(best_q_idx)
            }
            Algorithm::WorkloadAware => ready_queue.remove(0), // Handled by effective_algo above
        };

        // Context switch accounting
        if last_running_id != Some(tasks[selected_idx].id) {
            if last_running_id.is_some() {
                context_switches += 1;
            }
            last_running_id = Some(tasks[selected_idx].id);
            quantum_left = quantum;
        }

        if tasks[selected_idx].first_start_time.is_none() {
            tasks[selected_idx].first_start_time = Some(current_time);
            tasks[selected_idx].response_time = current_time - tasks[selected_idx].arrival_time;
        }

        tasks[selected_idx].state = TaskState::Running;

        // Execute task for 1 time unit
        if tasks[selected_idx].remaining_burst > 0 {
            tasks[selected_idx].remaining_burst -= 1;
        }
        if quantum_left > 0 {
            quantum_left -= 1;
        }
        current_time += 1;

        // Waiting time for all other ready tasks
        for &other_idx in ready_queue.iter() {
            tasks[other_idx].total_waiting_time += 1;
        }

        let task = &mut tasks[selected_idx];

        // Check if task completed
        if task.remaining_burst == 0 {
            task.state = TaskState::Completed;
            task.completion_time = Some(current_time);
            task.turnaround_time = current_time - task.arrival_time;
            completed_count += 1;
            last_running_id = None;
        } else {
            // Check for I/O blocking event
            let should_block = match task.io_freq {
                IoFreq::Frequent => task.remaining_burst % 2 == 0,
                IoFreq::Occasional => task.remaining_burst % 6 == 0,
                IoFreq::None => false,
            };

            if should_block {
                task.state = TaskState::Blocked;
                task.io_remaining = 3; // 3 time units I/O latency
                last_running_id = None;
            } else if effective_algo == Algorithm::RoundRobin && quantum_left == 0 {
                // Quantum expired: requeue at back
                task.state = TaskState::Ready;
                ready_queue.push(selected_idx);
                last_running_id = None;
            } else {
                // Continue running or keep in ready
                task.state = TaskState::Ready;
                ready_queue.insert(0, selected_idx); // Keep current task at front if quantum left
            }
        }
    }

    // Metrics calculation
    let mut total_resp = 0;
    let mut total_wait = 0;
    let mut total_turn = 0;
    let mut deadline_misses = 0;

    for t in &tasks {
        total_resp += t.response_time;
        total_wait += t.total_waiting_time;
        total_turn += t.turnaround_time;
        if let Some(dl) = t.deadline {
            if let Some(comp) = t.completion_time {
                if comp > dl {
                    deadline_misses += 1;
                }
            }
        }
    }

    let avg_response_time = total_resp as f32 / task_count as f32;
    let avg_waiting_time = total_wait as f32 / task_count as f32;
    let avg_turnaround_time = total_turn as f32 / task_count as f32;
    let throughput = if current_time > 0 {
        (task_count as f32 * 100.0) / current_time as f32
    } else {
        0.0
    };

    SchedulingResult {
        algorithm: algo,
        workload_class: trace.class,
        task_count,
        avg_response_time,
        avg_waiting_time,
        avg_turnaround_time,
        throughput,
        context_switches,
        deadline_misses,
        total_time: current_time,
        completed_tasks: completed_count,
        selection_reason: reason,
        chosen_algorithm: effective_algo,
    }
}

fn ready_query_non_preemptive<'a>(queue: &'a [usize], _tasks: &'a [Task]) -> impl Iterator<Item = (usize, usize)> + 'a {
    queue.iter().enumerate().map(|(q_idx, &t_idx)| (q_idx, t_idx))
}

fn effective_priority(task: &Task, _current_time: u32) -> u32 {
    // Aging: reduce priority number by 1 for every 10 time units of waiting time (capped at 1)
    let age_bonus = task.total_waiting_time / 10;
    if task.priority > age_bonus {
        (task.priority - age_bonus).max(1)
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workload_generation() {
        let trace = generate_workload(WorkloadClass::CpuBound, 8, 42);
        assert_eq!(trace.tasks.len(), 8);
        assert_eq!(trace.class, WorkloadClass::CpuBound);

        let charac = analyze_workload(&trace.tasks);
        assert_eq!(charac.classified_as, WorkloadClass::CpuBound);
        assert_eq!(charac.selected_algo, Algorithm::Fcfs);
    }

    #[test]
    fn test_scheduling_algorithms() {
        let trace = generate_workload(WorkloadClass::Interactive, 8, 123);
        let res_fcfs = simulate(Algorithm::Fcfs, &trace, 4);
        let res_rr = simulate(Algorithm::RoundRobin, &trace, 4);
        let res_wa = simulate(Algorithm::WorkloadAware, &trace, 4);

        assert_eq!(res_fcfs.completed_tasks, 8);
        assert_eq!(res_rr.completed_tasks, 8);
        assert_eq!(res_wa.completed_tasks, 8);

        // Round robin should provide better or equal responsiveness for interactive tasks
        assert!(res_rr.avg_response_time <= res_fcfs.avg_response_time + 1.0);
    }
}
