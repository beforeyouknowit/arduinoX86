/*
    ArduinoX86 Copyright 2022-2025 Daniel Balsom
    https://github.com/dbalsom/arduinoX86

    Permission is hereby granted, free of charge, to any person obtaining a
    copy of this software and associated documentation files (the “Software”),
    to deal in the Software without restriction, including without limitation
    the rights to use, copy, modify, merge, publish, distribute, sublicense,
    and/or sell copies of the Software, and to permit persons to whom the
    Software is furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in
    all copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
    FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
    DEALINGS IN THE SOFTWARE.
*/

use std::{collections::LinkedList, time::Instant};

use crate::{
    events::{GuiEvent, GuiEventQueue},
    structs::ScheduledEvent,
};

pub struct Scheduler {
    last_run: Instant,
    events:   LinkedList<ScheduledEvent>,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            last_run: Instant::now(),
            events:   LinkedList::new(),
        }
    }

    pub fn add_event(&mut self, event: ScheduledEvent) {
        self.events.push_back(event);
    }

    pub fn clear_events(&mut self) {
        self.events.clear();
    }

    pub fn remove_event_type(&mut self, event_type: &GuiEvent) {
        self.events.extract_if(|e| e.event == *event_type).for_each(drop);
    }

    pub fn replace_event_type(&mut self, event: ScheduledEvent) {
        self.remove_event_type(&event.event);
        self.add_event(event);
    }

    pub fn run(&mut self, events: &mut GuiEventQueue) {
        let elapsed = self.last_run.elapsed();
        self.last_run = Instant::now();

        let duration_millis = elapsed.as_millis() as u64;

        for event in self.events.iter_mut() {
            event.ms_accum += duration_millis;

            if event.ms_accum >= event.time {
                event.ms_accum -= event.time;

                match event.event {
                    GuiEvent::PollStatus => {
                        events.push(GuiEvent::PollStatus);
                    }
                    GuiEvent::RefreshMemory => {
                        events.push(GuiEvent::RefreshMemory);
                    }
                    _ => {}
                }
            }
        }
    }
}
