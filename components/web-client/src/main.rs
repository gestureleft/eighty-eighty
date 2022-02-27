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

    let handle_step_forward = {
        let state_history = state_history.clone();
        Callback::from(move |_| {
            let state_history_vals = &*state_history;
            let mut cpu = state_history[state_history.len() - 1];
            // FIXME - better error handling
            cpu.step().expect("Failed to step cpu");
            let mut new_history = state_history_vals.clone();
            new_history.push(cpu);
            state_history.set(new_history);

            log::debug!("Just set state history: {}", state_history.len());
        })
    };

    log::debug!("App function called!");

    let latest_cpu_state = (*state_history)[(*state_history).len() - 1];

    html! {
        <div class="col">
            <b>{"Assembly Input:"}</b>
            <input ondrop={handle_file_drop} type={"file"}/>
            <div class="row">
            <button>{"< Step Backward"}</button>
            <button>{"Reset"}</button>
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
