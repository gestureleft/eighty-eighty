use eighty_eighty::Cpu;
use yew::{function_component, html, use_state, Callback, Html, Properties};

#[derive(Properties, PartialEq)]
pub(crate) struct CpuStateProps<BusWriteCallback: FnMut(u8) + PartialEq> {
    pub(crate) cpu: Cpu<BusWriteCallback>,
}

fn make_row(memory: &[u8], row: u8, show_ascii: bool) -> Html {
    let mut iter = memory.iter();
    if row > 0 {
        iter.nth((row * 16 - 1).into());
    };
    iter.take(16)
        .map(|&i| html! { <div class="memory-cell memory-value">{if show_ascii { format!("{}", i as char) } else { format!("{:X}", i)} }</div> })
        .collect::<Html>()
}

#[function_component(CpuState)]
pub(crate) fn cpu_state<BusWriteCallback: FnMut(u8) + PartialEq>(
    CpuStateProps { cpu }: &CpuStateProps<BusWriteCallback>,
) -> Html {
    let show_ascii = use_state(|| false);

    let headers = (0..=0xfu8)
        .map(|i| html! { <div class="memory-cell memory-header">{format!("{:X}", i)}</div> })
        .collect::<Html>();

    let table_content = (0..=0xf)
        .map(|i| {
            html! {
                <>
                    <div class="memory-cell memory-header">{format!("{:03X}", i)}</div>
                    {make_row(cpu.memory(), i, *show_ascii)}
                </>
            }
        })
        .collect::<Html>();

    html! {
        <>
            <div class="cpu-state-container">
                <div class="row" style="margin-bottom: 1rem">
                    <div class="row items-base reg"><div class="reg-name">{"a: "}</div>{format!("{:02X}", cpu.a())}</div>
                    <div class="row items-base reg"><div class="reg-name">{"b: "}</div>{format!("{:02X}", cpu.b())}</div>
                    <div class="row items-base reg"><div class="reg-name">{"c: "}</div>{format!("{:02X}", cpu.c())}</div>
                    <div class="row items-base reg"><div class="reg-name">{"d: "}</div>{format!("{:02X}", cpu.d())}</div>
                    <div class="row items-base reg"><div class="reg-name">{"e: "}</div>{format!("{:02X}", cpu.e())}</div>
                </div>
                <div class="row">
                    <div class="row items-base reg"><div class="reg-name">{"h: "}</div>{format!("{:02X}", cpu.h())}</div>
                    <div class="row items-base reg"><div class="reg-name">{"l: "}</div>{format!("{:02X}", cpu.l())}</div>
                    <div class="row items-base reg"><div class="reg-name">{"pc: "}</div>{format!("{:04X}", cpu.pc())}</div>
                    <div class="row items-base reg"><div class="reg-name">{"sp: "}</div>{format!("{:04X}", cpu.sp())}</div>
                    <div class="row items-base reg"><div class="reg-name">{"halted: "}</div>{if cpu.halted() { "Yes" } else { "No" }.to_owned()}</div>
                </div>
            </div>
            <div class="my-md">
                <input checked={*show_ascii} id="toggle" type="checkbox" onclick={Callback::from(move |_| { show_ascii.set(!*show_ascii) })} />
                <label for="toggle">{"ASCII"}</label>
            </div>
            <div class="memory-container">
                <div />
                {headers}
                {table_content}
            </div>
        </>
    }
}
