/// TODO:
/// We want to encapsulate the Vec of CPUs and the current index into that vec
/// we are at in a single type that will be the app's state
///
/// We want to render an arrow to the instruction next to be executed
/// (cpu.pc === instruction.src_position)
use eighty_eighty::Cpu;
use wasm_bindgen::JsCast;
use web_sys::{DragEvent, File};
use yew::{function_component, html, use_state, Callback, UseStateHandle};

mod cpu_state;
mod instruction;

use cpu_state::CpuState;

fn get_first_file_from_drag_event(drag_event: DragEvent) -> Option<File> {
    drag_event.data_transfer()?.files()?.get(0)
}

fn handle_bus_val(i: u8) {
    println!("Val on bus: {}", i);
}

type CpuCallback = fn(u8) -> ();

#[function_component(App)]
fn app() -> Html {
    let state_history: UseStateHandle<Vec<Cpu<CpuCallback>>> =
        use_state(|| vec![Cpu::new(handle_bus_val as fn(u8) -> ())]);

    let handle_file_drop = {
        let state_history = state_history.clone();
        Callback::from(move |drag_event: DragEvent| {
            let file = get_first_file_from_drag_event(drag_event);
            if file.is_none() {
                return;
            };

            let state_history = state_history.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let file_array_buffer =
                    wasm_bindgen_futures::JsFuture::from(file.unwrap().array_buffer())
                        .await
                        .expect("Failed to get the file array buffer");

                log::debug!("Got file_array_buffer: {:?}", file_array_buffer);

                let array_buffer = file_array_buffer
                    .dyn_into::<js_sys::ArrayBuffer>()
                    .expect("File.array_buffer() didn't return an ArrayBuffer...");

                let array_buffer = js_sys::Uint8Array::new(&array_buffer);
                let vec = array_buffer.to_vec();

                let mut new_cpu = Cpu::new(handle_bus_val as fn(u8) -> ());

                let buffer_len = vec.len();

                new_cpu.load_into_memory(vec).unwrap_or_else(|_| {
                    panic!("Error loading buf of len {} into CPU memory", buffer_len)
                });

                state_history.set(vec![new_cpu]);
            });
        })
    };

    let latest_cpu_state = (*state_history)[(*state_history).len() - 1];

    let step_cpu = {
        let state_history = state_history.clone();
        move || {
            let mut cpu = (*state_history)[(*state_history).len() - 1];
            cpu.step().expect("Failed to step cpu");
            let mut new_state_history = (*state_history).clone();
            new_state_history.push(cpu);
            state_history.set(new_state_history);
        }
    };

    let handle_step_forward = Callback::from(move |_| {
        step_cpu();
    });

    let handle_step_backward = {
        let state_history = state_history.clone();
        Callback::from(move |_| {
            if (*state_history).len() > 1 {
                let mut new_state_history = (*state_history).clone();
                new_state_history.pop();
                state_history.set(new_state_history);
            }
        })
    };

    let handle_reset = {
        let state_history = state_history.clone();
        Callback::from(move |_| {
            state_history.set(vec![Cpu::new(handle_bus_val as fn(u8) -> ())]);
        })
    };

    let handle_run = {
        Callback::from(move |_| {
            let mut new_state_history = (*state_history).clone();
            let mut cpu = (*state_history)[(*state_history).len() - 1];
            while !cpu.halted() && new_state_history.len() < 1000 {
                cpu.step().expect("Failed to step cpu");
                new_state_history.push(cpu);
            }
            state_history.set(new_state_history);
        })
    };

    log::debug!("App function called!");

    html! {
        <div class="col">
            <b>{"Binary File:"}</b>
            <input ondrop={handle_file_drop} type={"file"}/>
            <div class="mt-md row">
                <button class="mr-lg" onclick={handle_run}>{"Run"}</button>
                <button onclick={handle_step_backward}>{"< Step Backward"}</button>
                <button onclick={handle_reset}>{"Reset"}</button>
                <button onclick={handle_step_forward}>{"Step Forward >"}</button>
            </div>
            <CpuState<CpuCallback> cpu={latest_cpu_state}/>
        </div>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());

    yew::start_app::<App>();
}
