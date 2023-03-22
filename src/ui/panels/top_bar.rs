use vizia::prelude::*;

use crate::state_system::{actions::TimelineAction, AppAction, StateSystem, WorkingState};
use crate::ui::generic_views::icon::*;
use crate::ui::generic_views::{
    icon, IconCode, ICON_PLAYER_PAUSE_FILLED, ICON_PLAYER_PLAY_FILLED, ICON_PLAYER_RECORD_FILLED,
    ICON_PLAYER_STOP_FILLED, ICON_REPEAT,
};
#[derive(Lens)]
pub struct TopBar {
    collapsed: bool,
    settings: bool,
}

impl TopBar {
    pub fn new(cx: &mut Context) -> Handle<Self> {
        Self { collapsed: false, settings: false }
            .build(cx, |cx| {
                // Checkbox::new(cx, TopBar::collapsed).on_toggle(|ex| ex.emit(TopBarEvent::ToggleExpanded));
                ControlGroup::new(cx).left(Stretch(1.0));
                VStack::new(cx, |cx|{
                    Icon::new(cx, ICON_SETTINGS).on_press(|ex| ex.emit(TopBarEvent::ToggleSettings));
                    Popup::new(cx, TopBar::settings, false, |cx|{
                        HStack::new(cx, |cx|{
                            Checkbox::new(cx, TopBar::collapsed.map(|flag| !*flag)).on_toggle(|ex| ex.emit(TopBarEvent::ToggleExpanded));
                            Label::new(cx, "Show Header Titles");
                        })
                        .width(Pixels(100.0))
                        .class("row2");
                    })
                    .on_blur(|ex| ex.emit(TopBarEvent::CloseSettings))
                    .right(Pixels(0.0)).class("settings");
                }).size(Auto);
            })
            .toggle_class("collapsed", TopBar::collapsed)
            .layout_type(LayoutType::Row)
    }
}

pub enum TopBarEvent {
    ToggleExpanded,
    ToggleSettings,
    CloseSettings,
}

impl View for TopBar {
    fn element(&self) -> Option<&'static str> {
        Some("topbar")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|topbar_event, _| match topbar_event {
            TopBarEvent::ToggleExpanded => {
                self.collapsed ^= true;
            }

            TopBarEvent::ToggleSettings => {
                self.settings ^= true;
            }

            TopBarEvent::CloseSettings => {
                self.settings = false;
            }
        })
    }
}

#[derive(Data, Clone, PartialEq, Eq)]
pub struct Control {
    visible: bool,
    icon: &'static str,
    label: &'static str,
}

#[derive(Data, Clone, PartialEq, Eq)]
pub enum ControlOrSeparator {
    Control(Control),
    Separator,
}

#[derive(Lens)]
pub struct ControlGroup {
    open_settings: bool,
    controls: Vec<ControlOrSeparator>,
    selected: Option<usize>,
    list_change: usize,
    // Derived state
    add_separator_disabled: bool,
    remove_disabled: bool,
    move_up_disabled: bool,
    move_down_disabled: bool,
}

impl ControlGroup {
    pub fn new(cx: &mut Context) -> Handle<Self> {
        Self {
            open_settings: false,
            list_change: 0,
            controls: vec![
                ControlOrSeparator::Control(Control {
                    visible: true,
                    icon: ICON_REPEAT,
                    label: "Loop",
                }),
                ControlOrSeparator::Separator,
                ControlOrSeparator::Control(Control {
                    visible: true,
                    icon: ICON_PLAYER_PLAY_FILLED,
                    label: "Play",
                }),
                ControlOrSeparator::Control(Control {
                    visible: true,
                    icon: ICON_PLAYER_STOP_FILLED,
                    label: "Stop",
                }),
                ControlOrSeparator::Control(Control {
                    visible: true,
                    icon: ICON_PLAYER_RECORD_FILLED,
                    label: "Record",
                }),
            ],
            selected: None,
            add_separator_disabled: true,
            remove_disabled: true,
            move_up_disabled: true,
            move_down_disabled: true,
            
        }
        .build(cx, |cx| {
            Label::new(cx, "TRANSPORT").class("title");
            Element::new(cx).class("horizontal_line");
            HStack::new(cx, |cx| {
                // Icon::new(cx, ICON_REPEAT);
                // Icon::new(cx, ICON_PLAYER_PLAY_FILLED);
                // Icon::new(cx, ICON_PLAYER_PAUSE_FILLED);
                // Icon::new(cx, ICON_PLAYER_RECORD_FILLED);
                Binding::new(cx, ControlGroup::list_change, |cx, controls| {
                    for (index, control_or_separator) in
                        ControlGroup::controls.get(cx).iter().enumerate()
                    {
                        let item = ControlGroup::controls.index(index);
                        match control_or_separator {
                            ControlOrSeparator::Control(control) => {
                                Icon::new(cx, control.icon).class("control1").toggle_class(
                                    "hidden",
                                    item.map(|control| {
                                        if let ControlOrSeparator::Control(control) = control {
                                            !control.visible
                                        } else {
                                            false
                                        }
                                    }),
                                );
                            }

                            ControlOrSeparator::Separator => {
                                Element::new(cx).class("vertical_separator").right(Pixels(8.0));
                            }
                        }
                    }
                });
                VStack::new(cx, |cx| {
                    Icon::new(cx, ICON_SETTINGS_PLUS)
                        .on_press(|ex| ex.emit(ControlGroupEvent::ToggleSettings));
                    Popup::new(cx, ControlGroup::open_settings, false, |cx| {
                        //Binding::new(cx, ControlGroup::controls, |cx, controls| {
                        HStack::new(cx, |cx| {
                            Button::new(
                                cx,
                                |ex| ex.emit(ControlGroupEvent::AddSeparator),
                                |cx| Label::new(cx, "Add Separator"),
                            ).child_left(Pixels(8.0)).child_right(Pixels(8.0)).disabled(ControlGroup::add_separator_disabled);

                            Button::new(
                                cx,
                                |ex| ex.emit(ControlGroupEvent::Remove),
                                |cx| Icon::new(cx, ICON_TRASH),
                            )
                            .disabled(ControlGroup::remove_disabled);

                            Button::new(
                                cx,
                                |ex| ex.emit(ControlGroupEvent::MoveUp),
                                |cx| Icon::new(cx, ICON_ARROW_UP),
                            )
                            .disabled(ControlGroup::move_up_disabled);

                            Button::new(
                                cx,
                                |ex| ex.emit(ControlGroupEvent::MoveDown),
                                |cx| Icon::new(cx, ICON_ARROW_DOWN),
                            ).disabled(ControlGroup::move_down_disabled);
                        })
                        .col_between(Pixels(4.0))
                        .height(Pixels(30.0));
                        Binding::new(cx, ControlGroup::list_change, |cx, _| {
                            println!("rebuild");
                            for (index, control) in
                                ControlGroup::controls.get(cx).iter().enumerate()
                            {
                                let item = ControlGroup::controls.index(index);
                                HStack::new(cx, |cx| match control {
                                    ControlOrSeparator::Control(control) => {
                                        Icon::new(
                                            cx,
                                            item.map(|control_or_separator| {
                                                if let ControlOrSeparator::Control(control) =
                                                    control_or_separator
                                                {
                                                    if control.visible {
                                                        ICON_EYE
                                                    } else {
                                                        ICON_EYE_CLOSED
                                                    }
                                                } else {
                                                    ICON_EYE
                                                }
                                            }),
                                        )
                                        .on_press(
                                            move |ex| {
                                                ex.emit(ControlGroupEvent::ToggleVisibility(index))
                                            },
                                        );
                                        Icon::new(cx, control.icon);
                                        Label::new(cx, control.label);
                                    }

                                    ControlOrSeparator::Separator => {
                                        Label::new(cx, "--- Separator Line ---").left(Stretch(1.0)).right(Stretch(1.0));
                                    }
                                })
                                .on_press(move |ex| ex.emit(ControlGroupEvent::Select(index)))
                                .toggle_class(
                                    "selected",
                                    ControlGroup::selected
                                        .map(move |selected| *selected == Some(index)),
                                )
                                .class("row");
                            }
                        });
                        //});
                    })
                    .on_blur(|ex| ex.emit(ControlGroupEvent::CloseSettings))
                    .class("settings");
                })
                .size(Auto);
            })
            .class("controls");
        })
    }
}

pub enum ControlGroupEvent {
    ToggleSettings,
    CloseSettings,
    ToggleVisibility(usize),
    MoveUp,
    MoveDown,
    Remove,
    Select(usize),
    AddSeparator,
}

impl View for ControlGroup {
    fn element(&self) -> Option<&'static str> {
        Some("control_group")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|control_group_event, meta| match control_group_event {
            ControlGroupEvent::ToggleSettings => {
                self.open_settings ^= true;
            }

            ControlGroupEvent::CloseSettings => {
                self.open_settings = false;
            }

            ControlGroupEvent::ToggleVisibility(index) => match &mut self.controls[*index] {
                ControlOrSeparator::Control(control) => {
                    control.visible ^= true;
                }

                _ => unreachable!(),
            },

            ControlGroupEvent::Select(index) => {
                self.selected = Some(*index);

                self.add_separator_disabled = false;
                self.remove_disabled = false;
                self.move_up_disabled = false;
                self.move_down_disabled = false;

                if *index == 0 {
                    self.move_up_disabled = true;
                } else if *index == self.controls.len() - 1 {
                    self.move_down_disabled = true;
                    self.add_separator_disabled = true;
                }

                if self.controls[*index] != ControlOrSeparator::Separator {
                    self.remove_disabled = true;
                }


            }

            ControlGroupEvent::AddSeparator => {
                if let Some(selected) = self.selected {
                    self.controls.insert(selected + 1, ControlOrSeparator::Separator);
                    self.list_change += 1;
                }
            }

            ControlGroupEvent::Remove => {
                if let Some(selected) = self.selected {
                    if let ControlOrSeparator::Separator = self.controls[selected] {
                        self.controls.remove(selected);
                        self.selected = None;
                        self.list_change += 1;
                    }
                }
            }

            ControlGroupEvent::MoveUp => {
                if let Some(selected) = self.selected {
                    if selected != 0 {
                        self.controls.swap(selected, selected - 1);
                        self.selected = Some(selected - 1);
                        self.list_change += 1;
                    }
                }
            }

            ControlGroupEvent::MoveDown => {
                if let Some(selected) = self.selected {
                    if selected != self.controls.len() - 1 {
                        self.controls.swap(selected, selected + 1);
                        self.selected = Some(selected + 1);
                        self.list_change += 1;
                    }
                }
            }

            _ => {}
        })
    }
}

pub struct Icon {}

impl Icon {
    pub fn new<'a, S: ToString>(cx: &'a mut Context, icon_label: impl Res<S>) -> Handle<'a, Self> {
        Self {}.build(cx, |cx| {}).text(icon_label)
    }
}

impl View for Icon {
    fn element(&self) -> Option<&'static str> {
        Some("icon")
    }
}

pub fn top_bar(cx: &mut Context) {
    const TOP_BAR_HEIGHT: f32 = 36.0;
    const TOP_BAR_CHILD_SPACE: f32 = 2.0;

    const TOOLBAR_GROUP_HEIGHT: f32 = 28.0;
    const MENU_SEPARATOR_PADDING: f32 = 1.0;
    const SEPARATOR_PADDING: f32 = 9.0;
    const LABEL_LR_PADDING: f32 = 5.0;

    const ICON_FRAME_SIZE: f32 = 26.0;
    const ICON_SIZE: f32 = 25.0;
    const SMALL_ICON_FRAME_SIZE: f32 = 20.0;
    const SMALL_ICON_SIZE: f32 = 18.0;

    HStack::new(cx, |cx| {
        MenuController::new(cx, false, |cx| {
            MenuStack::new_horizontal(cx, |cx| {
                Menu::new(
                    cx,
                    |cx| Label::new(cx, "File"),
                    |cx| {
                        MenuButton::new_simple(cx, "Open Project", |_| {});
                        MenuButton::new_simple(cx, "Save", |_| {});
                        MenuButton::new_simple(cx, "Save As", |_| {});
                    },
                );

                Menu::new(
                    cx,
                    |cx| Label::new(cx, "Edit").class("menu_bar_label"),
                    |cx| {
                        MenuButton::new_simple(cx, "TODO", |_| {});
                    },
                );

                Menu::new(
                    cx,
                    |cx| Label::new(cx, "View"),
                    |cx| {
                        MenuButton::new_simple(cx, "TODO", |_| {});
                    },
                );

                Menu::new(
                    cx,
                    |cx| Label::new(cx, "Help"),
                    |cx| {
                        MenuButton::new_simple(cx, "About", |_| {});
                    },
                );
            });
        })
        .top(Stretch(1.0))
        .bottom(Stretch(1.0));

        Element::new(cx)
            .left(Pixels(SEPARATOR_PADDING))
            .right(Pixels(SEPARATOR_PADDING))
            .class("top_bar_separator");

        HStack::new(cx, |cx| {
            Button::new(cx, |_| {}, |cx| icon(cx, IconCode::Undo, ICON_FRAME_SIZE, ICON_SIZE))
                .class("icon_btn");

            Element::new(cx).class("toolbar_group_separator");

            Button::new(cx, |_| {}, |cx| icon(cx, IconCode::Redo, ICON_FRAME_SIZE, ICON_SIZE))
                .class("icon_btn");
        })
        .class("toolbar_group")
        .height(Pixels(TOOLBAR_GROUP_HEIGHT))
        .width(Auto);

        HStack::new(cx, |cx| {
            Button::new(cx, |_| {}, |cx| icon(cx, IconCode::Save, ICON_FRAME_SIZE, ICON_SIZE))
                .class("icon_btn");
        })
        .class("toolbar_group")
        .left(Pixels(SEPARATOR_PADDING))
        .height(Pixels(TOOLBAR_GROUP_HEIGHT))
        .width(Auto);

        Element::new(cx).left(Pixels(SEPARATOR_PADDING)).class("top_bar_separator");

        HStack::new(cx, |cx| {
            Button::new(
                cx,
                |cx| {
                    cx.emit(AppAction::Timeline(TimelineAction::SetLoopActive(
                        !StateSystem::working_state
                            .then(WorkingState::transport_loop_active)
                            .get(cx),
                    )))
                },
                |cx| Label::new(cx, ICON_REPEAT).class("icons"),
            )
            .class("icon_btn")
            .toggle_class(
                "icon_btn_accent_toggled",
                StateSystem::working_state.then(WorkingState::transport_loop_active),
            );

            Element::new(cx).class("toolbar_group_separator");

            Button::new(
                cx,
                |cx| cx.emit(AppAction::Timeline(TimelineAction::TransportStop)),
                |cx| Label::new(cx, ICON_PLAYER_STOP_FILLED).class("icons"),
            )
            .class("icon_btn");

            Element::new(cx).class("toolbar_group_separator");

            // Label::new(cx, ICON_PLAYER_PLAY_FILLED).class("icons");
            // Binding::new(
            //     cx,
            //     StateSystem::working_state.then(WorkingState::transport_playing),
            //     |cx, transport_playing| {
            //         if transport_playing.get(cx) {
            Button::new(
                cx,
                |cx| cx.emit(AppAction::Timeline(TimelineAction::ToggleTransport)),
                |cx| {
                    Label::new(
                        cx,
                        StateSystem::working_state.then(WorkingState::transport_playing).map(
                            |playing| {
                                if *playing {
                                    ICON_PLAYER_PAUSE_FILLED
                                } else {
                                    ICON_PLAYER_PLAY_FILLED
                                }
                            },
                        ),
                    )
                    .class("icons")
                },
            )
            .class("icon_btn")
            .role(Role::ToggleButton);
            //         } else {
            //             Button::new(
            //                 cx,
            //                 |cx| cx.emit(AppAction::Timeline(TimelineAction::TransportPlay)),
            //                 |cx| icon(cx, IconCode::Play, ICON_FRAME_SIZE, ICON_SIZE),
            //             )
            //             .class("icon_btn");
            //         }
            //     },
            // );

            Element::new(cx).class("toolbar_group_separator");

            Button::new(cx, |_| {}, |cx| Label::new(cx, ICON_PLAYER_RECORD_FILLED).class("icons"))
                .class("record_btn");

            Element::new(cx).class("toolbar_group_separator");

            // TODO: Make this a functional widget.
            Label::new(cx, "1.1.1")
                .left(Pixels(35.0))
                .top(Stretch(1.0))
                .bottom(Stretch(1.0))
                .right(Pixels(LABEL_LR_PADDING));
        })
        .class("toolbar_group");

        Element::new(cx)
            .right(Pixels(SEPARATOR_PADDING))
            .left(Stretch(1.0))
            .class("top_bar_separator");

        HStack::new(cx, |cx| {
            // TODO: Make this a functional widget.
            Label::new(cx, "BPM")
                .top(Stretch(1.0))
                .bottom(Stretch(1.0))
                .left(Pixels(LABEL_LR_PADDING))
                .right(Pixels(LABEL_LR_PADDING))
                .class("toolbar_group_dimmed_label");

            //Element::new(cx).class("toolbar_group_separator");

            Label::new(cx, "120.000")
                .top(Stretch(1.0))
                .bottom(Stretch(1.0))
                .left(Pixels(LABEL_LR_PADDING))
                .right(Pixels(LABEL_LR_PADDING));
        })
        .class("toolbar_group")
        .height(Pixels(TOOLBAR_GROUP_HEIGHT))
        .width(Auto)
        .right(Pixels(SEPARATOR_PADDING));

        HStack::new(cx, |cx| {
            Button::new(
                cx,
                |_| {},
                |cx| {
                    HStack::new(cx, |cx| {
                        Label::new(cx, "TAP")
                            .top(Stretch(1.0))
                            .bottom(Stretch(1.0))
                            .right(Pixels(LABEL_LR_PADDING));
                    })
                    .child_left(Pixels(LABEL_LR_PADDING))
                    .child_right(Pixels(LABEL_LR_PADDING))
                },
            )
            .class("icon_btn")
            .top(Stretch(1.0))
            .bottom(Stretch(1.0))
            .height(Pixels(TOOLBAR_GROUP_HEIGHT - 2.0));
        })
        .width(Auto)
        .height(Pixels(TOOLBAR_GROUP_HEIGHT))
        .class("toolbar_group")
        .right(Pixels(SEPARATOR_PADDING));

        Element::new(cx).right(Pixels(SEPARATOR_PADDING)).class("top_bar_separator");

        HStack::new(cx, |cx| {
            Label::new(cx, "4 / 4")
                .class("time_signature_text")
                .top(Stretch(1.0))
                .bottom(Stretch(1.0))
                .left(Pixels(LABEL_LR_PADDING))
                .right(Pixels(LABEL_LR_PADDING));
        })
        .width(Auto)
        .class("toolbar_group")
        .height(Pixels(TOOLBAR_GROUP_HEIGHT))
        .right(Pixels(SEPARATOR_PADDING));

        HStack::new(cx, |cx| {
            Button::new(
                cx,
                |_| {},
                |cx| {
                    HStack::new(cx, |cx| {
                        Label::new(cx, "GRV")
                            .top(Stretch(1.0))
                            .bottom(Stretch(1.0))
                            .right(Pixels(LABEL_LR_PADDING));
                        icon(cx, IconCode::Menu, SMALL_ICON_FRAME_SIZE, SMALL_ICON_SIZE)
                            .top(Stretch(1.0))
                            .bottom(Stretch(1.0));
                    })
                    .child_left(Pixels(LABEL_LR_PADDING))
                    .child_right(Pixels(LABEL_LR_PADDING))
                },
            )
            .class("icon_btn")
            .top(Stretch(1.0))
            .bottom(Stretch(1.0))
            .height(Pixels(TOOLBAR_GROUP_HEIGHT - 2.0));
        })
        .width(Auto)
        .height(Pixels(TOOLBAR_GROUP_HEIGHT))
        .class("toolbar_group")
        .right(Pixels(SEPARATOR_PADDING));
    })
    .height(Pixels(TOP_BAR_HEIGHT))
    .child_space(Pixels(TOP_BAR_CHILD_SPACE))
    .class("top_bar");
}
