use eighty_eighty::Instruction;
use yew::{function_component, html, Html, Properties};

#[derive(Properties, PartialEq)]
pub(crate) struct InstructionPaneProps {
    pub(crate) instructions: Vec<Instruction>,
}

#[function_component(InstructionPane)]
pub(crate) fn instruction_pane(
    InstructionPaneProps { instructions }: &InstructionPaneProps,
) -> Html {
    html! { <div class="instruction-pane col">
    {instructions.iter().enumerate().map(|(index, instr)| html! { <div class="row instruction-row"><div class="mr-md">{index + 1}</div> {instr}</div> }).collect::<Html>()}
    </div> }
}
