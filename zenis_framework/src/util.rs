use zenis_discord::*;

pub fn make_multiple_rows(buttons: Vec<ButtonBuilder>) -> Vec<ActionRowBuilder> {
    let mut rows = vec![];

    for i in (0..buttons.len()).step_by(3) {
        let mut row = ActionRowBuilder::new();
        for j in 0..3 {
            let Some(button) = buttons.get(i + j) else {
                break;
            };

            row = row.add_button(button.clone());
        }

        rows.push(row);
    }

    rows
}
