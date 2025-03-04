use std::ops::Deref;
use std::rc::Rc;
use itertools::Itertools;
use piston_window::*;
use piston_window::Key::Y;
use piston_window::types::Color;
use crate::runtime::simulation::forward::ForwardChainNode;

const BRANCH_Y_MARGIN: f64 = 12.0;
const BRANCH_X_MARGIN: f64 = 20.0;
const START_NODE_X: f64 = 8.0;
const START_NODE_Y: f64 = 8.0;
const NODE_WIDTH: f64 = 300.0;
const NODE_HEIGHT: f64 = 40.0;

fn draw_text(
    ctx: &Context,
    graphics: &mut G2d,
    glyphs: &mut Glyphs,
    color: Color,
    pos: [f64; 2],
    text: &str,
) {
    text::Text::new_color(color, 10)
        .draw(
            text,
            glyphs,
            &ctx.draw_state,
            ctx.transform.trans(pos[0], pos[1]),
            graphics,
        )
        .unwrap();
}

fn visualize_node(node: &Rc<ForwardChainNode>, index: usize, x: f64, y: f64, scroll_y: f64, depth: usize, selected_node: &Vec<usize>, context: &Context, graphics: &mut G2d, glyphs: &mut Glyphs) {
    if y > context.get_view_size()[1] {
        return;
    }

    let is_selected = matches!(selected_node.get(depth), Some(i) if i == &index);
    let color = if is_selected {
        [0.56, 0.93, 0.56, 1.0]
    }
    else if node.is_in_goal_path {
        [0.68, 0.85, 0.90, 1.0]
    }
    else {
        [1.0, 0.957, 0.722, 1.0]
    };

    rectangle(color, [x, y, NODE_WIDTH, NODE_HEIGHT], context.transform, graphics);
    draw_text(context, graphics, glyphs, [0.0, 0.0, 0.0, 1.0].into(), [x + 2.0, y + NODE_HEIGHT / 2.0 + 2.0], &format!("{}: {}", node.min_goal_depth, &node.command));

    if is_selected && selected_node.len() - 1 > depth {
        for (i, node) in node.children.iter().enumerate() {
            visualize_node(node, i, START_NODE_X + ((depth + 1) as f64 * (BRANCH_X_MARGIN + NODE_WIDTH)), START_NODE_Y + (i as f64 * (BRANCH_Y_MARGIN + NODE_HEIGHT)) - scroll_y, scroll_y, depth + 1, selected_node, context, graphics, glyphs);
        }
    }

    /*for (i, node) in node.children.iter().enumerate() {

    }*/
}

fn sort_tree(tree: &Vec<Rc<ForwardChainNode>>, depth: usize) -> Vec<Rc<ForwardChainNode>> {
    if depth >= 5 {
        return vec![];
    }
    tree.iter().sorted_by_key(|node| node.min_goal_depth).map(|node| Rc::new(ForwardChainNode {
        children: sort_tree(&node.children, depth + 1),
        ..node.deref().clone()
    })).collect()
}

pub fn visualize_forward_chaining(tree: &Vec<Rc<ForwardChainNode>>) {
    let tree = sort_tree(tree, 0);
    let mut window: PistonWindow = WindowSettings::new("Forward chaining visualization", [800, 800])
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut glyphs = window.load_font("Roboto-Regular.ttf").unwrap();
    let mut selected_node = vec![0_usize];
    let mut scroll_y = 0_f64;

    while let Some(event) = window.next() {
        match &event {
            Event::Input(inp, _) => match inp {
                Input::Button(ButtonArgs{ button: Button::Keyboard(Key::Up), state: ButtonState::Press, .. }) => {
                    let mut current_select = selected_node.last_mut().unwrap();
                    if *current_select > 0 {
                        *current_select -= 1;
                    }

                    if (START_NODE_Y + (*current_select as f64 * (BRANCH_Y_MARGIN + NODE_HEIGHT))) - scroll_y < 0.0 {
                        scroll_y -= BRANCH_Y_MARGIN + NODE_HEIGHT;
                    }
                }
                Input::Button(ButtonArgs{ button: Button::Keyboard(Key::Down), state: ButtonState::Press, .. }) => {
                    let mut current_select = selected_node.last_mut().unwrap();
                    *current_select += 1;

                    if START_NODE_Y + ((*current_select + 1) as f64 * (BRANCH_Y_MARGIN + NODE_HEIGHT)) - scroll_y > window.size().height {
                        scroll_y += BRANCH_Y_MARGIN + NODE_HEIGHT;
                    }
                }
                Input::Button(ButtonArgs{ button: Button::Keyboard(Key::Return), state: ButtonState::Press, .. }) => {
                    selected_node.push(0);
                    scroll_y = 0.0;
                }
                Input::Button(ButtonArgs{ button: Button::Keyboard(Key::Backspace), state: ButtonState::Press, .. }) => {
                    if selected_node.len() > 1 {
                        selected_node.pop();
                        scroll_y = 0.0;
                    }
                }
                Input::Button(ButtonArgs {button: Button::Mouse(MouseButton::Left), ..}) => {}
                _ => {}
            }
            _ => {}
        }

        window.draw_2d(&event, |context, graphics, _device| {
            clear([1.0; 4], graphics);

            for (i, node) in tree.iter().enumerate() {
                visualize_node(node, i, START_NODE_X, START_NODE_Y + (i as f64 * (BRANCH_Y_MARGIN + NODE_HEIGHT)) - scroll_y, scroll_y, 0, &selected_node, &context, graphics, &mut glyphs);
            }
            glyphs.factory.encoder.flush(_device);
        });
    }
}