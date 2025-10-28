#![allow(unused)]
use crate::components::loggedin::Action;
use crate::environment::types::{AppEvent, TimelineDirection};
use crate::environment::{Environment, OpenWindowState};
use crate::loc;
use crate::widgets::*;
use dioxus::prelude::*;
use std::rc::Rc;
use std::str::FromStr;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PreferencesChange {
    Direction,
    PostWindow,
}

#[derive(Clone, PartialEq)]
pub struct PreferencesWindowState {}

impl PreferencesWindowState {
    pub fn new() -> Self {
        Self {}
    }
}

impl OpenWindowState for PreferencesWindowState {
    type Action = PreferencesChange;

    fn window(
        &self,
        environment: &Environment,
        _receiver: flume::Receiver<AppEvent>,
        parent_handler: Rc<dyn Fn(Self::Action)>,
    ) -> Element {
        let Ok(current) = environment.settings.config() else {
            return rsx! {
                h3 { "An Error Occurred" }
            };
        };

        let direction = current.direction;
        let inline_postwindow = current.post_window_inline;
        let env1 = environment.clone();
        let env2 = environment.clone();
        let handler1 = parent_handler.clone();
        let handler2 = parent_handler.clone();

        rsx! {
            div {
                class: "settings-container",
                VStack {
                    class: "gap-4",
                    TimelineSetting {
                        direction: direction,
                        onchange: move |direction| {
                            if let Ok(mut current) = env1.settings.config() {
                                current.direction = direction;
                                std::mem::drop(env1.settings.set_config(&current));
                                handler1(PreferencesChange::Direction);
                            }
                        }
                    }
                    PostInlineSetting {
                        open_inline: inline_postwindow,
                        onchange: move |v| {
                            if let Ok(mut current) = env2.settings.config() {
                                current.post_window_inline = v;
                                std::mem::drop(env2.settings.set_config(&current));
                                handler2(PreferencesChange::PostWindow);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
struct TimelineDirectionOption {
    direction: TimelineDirection,
    selected: bool,
}

impl crate::widgets::Segment for TimelineDirectionOption {
    fn id(&self) -> u64 {
        match self.direction {
            TimelineDirection::NewestTop => 0,
            TimelineDirection::NewestBottom => 1,
        }
    }

    fn label(&self) -> String {
        match self.direction {
            TimelineDirection::NewestTop => "Newest on Top".to_string(),
            TimelineDirection::NewestBottom => "Newest on Bottom".to_string(),
        }
    }

    fn selected(&self) -> bool {
        self.selected
    }

    fn dot(&self) -> bool {
        false
    }
}

#[component]
fn TimelineSetting(
    direction: TimelineDirection,
    onchange: EventHandler<TimelineDirection>,
) -> Element {
    let items = vec![
        TimelineDirectionOption {
            direction: TimelineDirection::NewestTop,
            selected: direction == TimelineDirection::NewestTop,
        },
        TimelineDirectionOption {
            direction: TimelineDirection::NewestBottom,
            selected: direction == TimelineDirection::NewestBottom,
        },
    ];

    rsx! {
        VStack {
            class: "gap-1",
            h4 { "Timeline Direction" }
            SegmentedControl {
                items: items,
                onclick: move |item: TimelineDirectionOption| onchange.call(item.direction)
            }
        }
    }
}

#[component]
fn PostInlineSetting(open_inline: bool, onchange: EventHandler<bool>) -> Element {
    rsx! {
        VStack {
            class: "gap-1",
            HStack {
                class: "gap-4 items-center",
                Checkbox {
                    checked: open_inline,
                    onchange: move |v| onchange.call(v)
                }
                h4 { "Open posts in a new window" }
            }
            p {
                class: "text-sm text-gray-500",
                "When disabled, posts will open inline"
            }
        }
    }
}

impl FromStr for TimelineDirection {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "top" => Ok(TimelineDirection::NewestTop),
            "bottom" => Ok(TimelineDirection::NewestBottom),
            _ => Err("Unknown Direction".to_string()),
        }
    }
}
