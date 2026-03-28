use input_event::{
    Event, KeyboardEvent, PointerEvent,
    scancode::Linux::{
        self, KeyLeftAlt, KeyLeftCtrl, KeyLeftMeta, KeyLeftShift, KeyRightCtrl, KeyRightShift,
        KeyRightalt, KeyRightmeta,
    },
};
use lan_mouse_ipc::{ClientPlatform, InputProfile, ScrollMode, ShortcutMode};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LocalPlatform {
    Windows,
    Macos,
    Linux,
}

#[cfg(target_os = "windows")]
const LOCAL_PLATFORM: LocalPlatform = LocalPlatform::Windows;
#[cfg(target_os = "macos")]
const LOCAL_PLATFORM: LocalPlatform = LocalPlatform::Macos;
#[cfg(all(unix, not(target_os = "macos")))]
const LOCAL_PLATFORM: LocalPlatform = LocalPlatform::Linux;

#[derive(Debug, Clone, Copy, Default)]
struct ModifierState {
    ctrl: bool,
    alt: bool,
    meta: bool,
    shift: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct EventTransformer {
    profile: InputProfile,
    physical: ModifierState,
    emitted: ModifierState,
}

impl EventTransformer {
    pub(crate) fn new(profile: InputProfile) -> Self {
        Self {
            profile,
            physical: ModifierState::default(),
            emitted: ModifierState::default(),
        }
    }

    pub(crate) fn update_profile(&mut self, profile: InputProfile) {
        self.profile = profile;
    }

    pub(crate) fn transform(&mut self, event: Event) -> Vec<Event> {
        match event {
            Event::Pointer(pointer_event) => {
                vec![Event::Pointer(self.transform_pointer(pointer_event))]
            }
            Event::Keyboard(keyboard_event) => self.transform_keyboard(keyboard_event),
        }
    }

    fn transform_pointer(&self, event: PointerEvent) -> PointerEvent {
        match event {
            PointerEvent::Axis { time, axis, value } => PointerEvent::Axis {
                time,
                axis,
                value: self.transform_scroll_value(axis, value, false),
            },
            PointerEvent::AxisDiscrete120 { axis, value } => PointerEvent::AxisDiscrete120 {
                axis,
                value: self.transform_scroll_value(axis, value as f64, true) as i32,
            },
            other => other,
        }
    }

    fn transform_scroll_value(&self, axis: u8, value: f64, discrete: bool) -> f64 {
        let scale = match axis {
            1 => self.profile.scroll_scale_x,
            _ => self.profile.scroll_scale_y,
        } as f64;
        let direction = match self.profile.scroll_mode {
            ScrollMode::Physical => 1.0,
            ScrollMode::TargetNative => {
                if should_invert_scroll(self.profile.source_platform, LOCAL_PLATFORM) {
                    -1.0
                } else {
                    1.0
                }
            }
        };
        let transformed = value * direction * scale;
        if discrete {
            transformed.round()
        } else {
            transformed
        }
    }

    fn transform_keyboard(&mut self, event: KeyboardEvent) -> Vec<Event> {
        let KeyboardEvent::Key { time, key, state } = event else {
            return vec![Event::Keyboard(event)];
        };

        let Some(key) = Linux::try_from(key).ok() else {
            return vec![Event::Keyboard(KeyboardEvent::Key { time, key, state })];
        };

        if let Some(updated) = apply_modifier(&mut self.physical, key, state) {
            let desired = desired_modifier_state(self.profile.clone(), self.physical, None);
            return self.emit_modifier_delta(time, desired, Some(updated));
        }

        let desired = desired_modifier_state(self.profile.clone(), self.physical, Some(key));
        let mut events = self.emit_modifier_delta(time, desired, None);
        events.push(Event::Keyboard(KeyboardEvent::Key {
            time,
            key: key as u32,
            state,
        }));
        if state == 0 {
            let desired = desired_modifier_state(self.profile.clone(), self.physical, None);
            events.extend(self.emit_modifier_delta(time, desired, None));
        }
        events
    }

    fn emit_modifier_delta(
        &mut self,
        time: u32,
        desired: ModifierState,
        changed_key: Option<Linux>,
    ) -> Vec<Event> {
        let mut events = Vec::new();

        for (active, current, key) in [
            (
                desired.ctrl,
                self.emitted.ctrl,
                changed_key
                    .filter(|key| matches!(key, KeyLeftCtrl | KeyRightCtrl))
                    .unwrap_or(KeyLeftCtrl),
            ),
            (
                desired.alt,
                self.emitted.alt,
                changed_key
                    .filter(|key| matches!(key, KeyLeftAlt | KeyRightalt))
                    .unwrap_or(KeyLeftAlt),
            ),
            (
                desired.meta,
                self.emitted.meta,
                changed_key
                    .filter(|key| matches!(key, KeyLeftMeta | KeyRightmeta))
                    .unwrap_or(KeyLeftMeta),
            ),
            (
                desired.shift,
                self.emitted.shift,
                changed_key
                    .filter(|key| matches!(key, KeyLeftShift | KeyRightShift))
                    .unwrap_or(KeyLeftShift),
            ),
        ] {
            if active == current {
                continue;
            }
            let key = canonicalize_modifier_key(key);
            events.push(Event::Keyboard(KeyboardEvent::Key {
                time,
                key: key as u32,
                state: u8::from(active),
            }));
        }

        self.emitted = desired;
        events
    }
}

fn canonicalize_modifier_key(key: Linux) -> Linux {
    match key {
        KeyLeftCtrl | KeyRightCtrl => KeyLeftCtrl,
        KeyLeftAlt | KeyRightalt => KeyLeftAlt,
        KeyLeftMeta | KeyRightmeta => KeyLeftMeta,
        KeyLeftShift | KeyRightShift => KeyLeftShift,
        _ => key,
    }
}

fn apply_modifier(state: &mut ModifierState, key: Linux, pressed: u8) -> Option<Linux> {
    let pressed = pressed != 0;
    match key {
        KeyLeftCtrl | KeyRightCtrl => {
            state.ctrl = pressed;
            Some(key)
        }
        KeyLeftAlt | KeyRightalt => {
            state.alt = pressed;
            Some(key)
        }
        KeyLeftMeta | KeyRightmeta => {
            state.meta = pressed;
            Some(key)
        }
        KeyLeftShift | KeyRightShift => {
            state.shift = pressed;
            Some(key)
        }
        _ => None,
    }
}

fn desired_modifier_state(
    profile: InputProfile,
    physical: ModifierState,
    key: Option<Linux>,
) -> ModifierState {
    let mut desired = physical;
    if profile.shortcut_mode == ShortcutMode::SourceNative && is_primary_shortcut(key) {
        match (profile.source_platform, LOCAL_PLATFORM) {
            (ClientPlatform::Windows | ClientPlatform::Linux, LocalPlatform::Macos) => {
                if physical.ctrl {
                    desired.ctrl = false;
                    desired.meta = true;
                }
            }
            (ClientPlatform::Macos, LocalPlatform::Windows | LocalPlatform::Linux) => {
                if physical.meta {
                    desired.meta = false;
                    desired.ctrl = true;
                }
            }
            _ => {}
        }
    }
    desired
}

fn is_primary_shortcut(key: Option<Linux>) -> bool {
    key.is_some_and(|key| {
        !matches!(
            key,
            KeyLeftCtrl
                | KeyRightCtrl
                | KeyLeftAlt
                | KeyRightalt
                | KeyLeftMeta
                | KeyRightmeta
                | KeyLeftShift
                | KeyRightShift
        )
    })
}

fn should_invert_scroll(source: ClientPlatform, target: LocalPlatform) -> bool {
    matches!(
        (source, target),
        (
            ClientPlatform::Windows | ClientPlatform::Linux,
            LocalPlatform::Macos
        ) | (
            ClientPlatform::Macos,
            LocalPlatform::Windows | LocalPlatform::Linux
        )
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use input_event::scancode::Linux::{KeyC, KeyLeftCtrl, KeyLeftMeta};

    fn profile(source_platform: ClientPlatform) -> InputProfile {
        InputProfile {
            source_platform,
            shortcut_mode: ShortcutMode::SourceNative,
            scroll_mode: ScrollMode::TargetNative,
            ..Default::default()
        }
    }

    #[test]
    fn physical_scroll_mode_keeps_direction() {
        let mut transformer = EventTransformer::new(InputProfile::default());
        let events = transformer.transform(Event::Pointer(PointerEvent::AxisDiscrete120 {
            axis: 0,
            value: 120,
        }));
        assert_eq!(
            events,
            vec![Event::Pointer(PointerEvent::AxisDiscrete120 {
                axis: 0,
                value: 120
            })]
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn windows_scroll_inverts_on_macos() {
        let mut transformer = EventTransformer::new(profile(ClientPlatform::Windows));
        let events = transformer.transform(Event::Pointer(PointerEvent::AxisDiscrete120 {
            axis: 0,
            value: 120,
        }));
        assert_eq!(
            events,
            vec![Event::Pointer(PointerEvent::AxisDiscrete120 {
                axis: 0,
                value: -120
            })]
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn windows_ctrl_shortcut_maps_to_command_on_macos() {
        let mut transformer = EventTransformer::new(profile(ClientPlatform::Windows));
        let ctrl_down = transformer.transform(Event::Keyboard(KeyboardEvent::Key {
            time: 0,
            key: KeyLeftCtrl as u32,
            state: 1,
        }));
        assert_eq!(
            ctrl_down,
            vec![Event::Keyboard(KeyboardEvent::Key {
                time: 0,
                key: KeyLeftCtrl as u32,
                state: 1
            })]
        );

        let c_down = transformer.transform(Event::Keyboard(KeyboardEvent::Key {
            time: 1,
            key: KeyC as u32,
            state: 1,
        }));
        assert_eq!(
            c_down,
            vec![
                Event::Keyboard(KeyboardEvent::Key {
                    time: 1,
                    key: KeyLeftCtrl as u32,
                    state: 0
                }),
                Event::Keyboard(KeyboardEvent::Key {
                    time: 1,
                    key: KeyLeftMeta as u32,
                    state: 1
                }),
                Event::Keyboard(KeyboardEvent::Key {
                    time: 1,
                    key: KeyC as u32,
                    state: 1
                })
            ]
        );

        let c_up = transformer.transform(Event::Keyboard(KeyboardEvent::Key {
            time: 2,
            key: KeyC as u32,
            state: 0,
        }));
        assert_eq!(
            c_up,
            vec![
                Event::Keyboard(KeyboardEvent::Key {
                    time: 2,
                    key: KeyC as u32,
                    state: 0
                }),
                Event::Keyboard(KeyboardEvent::Key {
                    time: 2,
                    key: KeyLeftCtrl as u32,
                    state: 1
                }),
                Event::Keyboard(KeyboardEvent::Key {
                    time: 2,
                    key: KeyLeftMeta as u32,
                    state: 0
                })
            ]
        );
    }
}
