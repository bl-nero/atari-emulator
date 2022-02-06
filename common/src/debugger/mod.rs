pub mod adapter;
mod protocol;

use crate::debugger::adapter::DebugAdapter;
use crate::debugger::adapter::DebugAdapterError;
use crate::debugger::adapter::DebugAdapterResult;
use crate::debugger::protocol::IncomingMessage;
use crate::debugger::protocol::OutgoingMessage;
use debugserver_types::AttachResponse;
use debugserver_types::InitializeResponse;
use debugserver_types::InitializedEvent;
use debugserver_types::SetExceptionBreakpointsResponse;
use debugserver_types::StoppedEvent;
use debugserver_types::StoppedEventBody;
use std::sync::mpsc::TryRecvError;

/// A debugger for 6502-based machines. Uses Debug Adapter Protocol internally
/// to communicate with a debugger UI.
///
/// Note: some of the code here is absolutely horrible, because it's driven by
/// how bad the autogenerated [`debugserver_types`] play with our need of
/// generalizing over these.
/// TODO: Consider using raw JSON values or writing our own types. We won't need
/// many of them.
pub struct Debugger<A: DebugAdapter> {
    adapter: A,
    sequence_number: i64,
}

impl<A: DebugAdapter> Debugger<A> {
    pub fn new(adapter: A) -> Self {
        Self {
            adapter,
            sequence_number: 0,
        }
    }

    pub fn process_meessages(&mut self) {
        loop {
            match self.adapter.try_receive_message() {
                Ok(IncomingMessage::Initialize(req)) => {
                    self.send_message(OutgoingMessage::Initialize(InitializeResponse {
                        seq: -1,
                        request_seq: req.seq,
                        type_: "response".into(),
                        command: "initialize".into(),
                        success: true,
                        message: None,
                        body: None,
                    }))
                    .unwrap();
                    self.send_message(OutgoingMessage::Initialized(InitializedEvent {
                        seq: -1,
                        type_: "event".into(),
                        event: "initialized".into(),
                        body: None,
                    }))
                    .unwrap();
                }
                Ok(IncomingMessage::SetExceptionBreakpoints(req)) => {
                    self.send_message(OutgoingMessage::SetExceptionBreakpoints(
                        SetExceptionBreakpointsResponse {
                            seq: -1,
                            request_seq: req.seq,
                            type_: "response".into(),
                            command: "set_exception_breakpoints".into(),
                            success: true,
                            message: None,
                            body: None,
                        },
                    ))
                    .unwrap();
                }
                Ok(IncomingMessage::Attach(req)) => {
                    self.send_message(OutgoingMessage::Attach(AttachResponse {
                        seq: -1,
                        request_seq: req.seq,
                        type_: "response".into(),
                        command: "attach".into(),
                        success: true,
                        message: None,
                        body: None,
                    }))
                    .unwrap();
                    self.send_message(OutgoingMessage::Stopped(StoppedEvent {
                        seq: -1,
                        type_: "event".into(),
                        event: "stopped".into(),
                        body: StoppedEventBody {
                            reason: "entry".into(),
                            description: None,
                            thread_id: None,
                            preserve_focus_hint: None,
                            text: None,
                            all_threads_stopped: None,
                        },
                    }))
                    .unwrap();
                }
                Ok(other) => eprintln!("Unsupported message: {:?}", other),
                Err(DebugAdapterError::TryRecvError(TryRecvError::Empty)) => return,
                Err(e) => panic!("{}", e),
            }
        }
    }

    fn send_message(&mut self, mut message: OutgoingMessage) -> DebugAdapterResult<()> {
        use OutgoingMessage::*;
        match &mut message {
            Initialize(msg) => msg.seq = self.next_sequence_number(),
            Attach(msg) => msg.seq = self.next_sequence_number(),
            Next(msg) => msg.seq = self.next_sequence_number(),
            Evaluate(msg) => msg.seq = self.next_sequence_number(),

            Initialized(msg) => msg.seq = self.next_sequence_number(),
            SetExceptionBreakpoints(msg) => msg.seq = self.next_sequence_number(),
            Stopped(msg) => msg.seq = self.next_sequence_number(),
        }
        return self.adapter.send_message(message);
    }

    fn next_sequence_number(&mut self) -> i64 {
        self.sequence_number += 1;
        return self.sequence_number;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::debugger::adapter::DebugAdapterResult;
    use debugserver_types::AttachRequest;
    use debugserver_types::AttachRequestArguments;
    use debugserver_types::InitializeRequest;
    use debugserver_types::InitializeRequestArguments;
    use debugserver_types::SetExceptionBreakpointsArguments;
    use debugserver_types::SetExceptionBreakpointsRequest;
    use debugserver_types::SetExceptionBreakpointsResponse;
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::rc::Rc;

    fn initialize_request() -> DebugAdapterResult<IncomingMessage> {
        Ok(IncomingMessage::Initialize(InitializeRequest {
            seq: 5,
            type_: "request".into(),
            command: "initialize".into(),
            arguments: InitializeRequestArguments {
                client_id: Some("vscode".into()),
                client_name: Some("Visual Studio Code".into()),
                adapter_id: "steampunk-6502".into(),
                path_format: Some("path".into()),
                lines_start_at_1: Some(true),
                columns_start_at_1: Some(true),
                supports_variable_type: Some(true),
                supports_variable_paging: Some(true),
                supports_run_in_terminal_request: Some(true),
                locale: Some("en-us".into()),
            },
        }))
    }

    fn set_exception_breakpoints_request() -> DebugAdapterResult<IncomingMessage> {
        Ok(IncomingMessage::SetExceptionBreakpoints(
            SetExceptionBreakpointsRequest {
                seq: 6,
                type_: "request".into(),
                command: "setExceptionBreakpoints".into(),
                arguments: SetExceptionBreakpointsArguments {
                    filters: vec![],
                    exception_options: None,
                },
            },
        ))
    }

    fn attach_request() -> DebugAdapterResult<IncomingMessage> {
        Ok(IncomingMessage::Attach(AttachRequest {
            seq: 8,
            type_: "request".into(),
            command: "attach".into(),
            arguments: AttachRequestArguments::default(),
        }))
    }

    #[derive(Default)]
    struct FakeDebugAdapterInternals {
        receiver_queue: VecDeque<DebugAdapterResult<IncomingMessage>>,
        sender_queue: VecDeque<OutgoingMessage>,
    }

    fn push_incoming(
        adapter_internals: &RefCell<FakeDebugAdapterInternals>,
        message: DebugAdapterResult<IncomingMessage>,
    ) {
        adapter_internals
            .borrow_mut()
            .receiver_queue
            .push_back(message);
    }

    fn pop_outgoing(
        adapter_internals: &RefCell<FakeDebugAdapterInternals>,
    ) -> Option<OutgoingMessage> {
        adapter_internals.borrow_mut().sender_queue.pop_front()
    }

    #[derive(Default)]
    struct FakeDebugAdapter {
        internals: Rc<RefCell<FakeDebugAdapterInternals>>,
    }

    impl FakeDebugAdapter {
        fn new() -> (Self, Rc<RefCell<FakeDebugAdapterInternals>>) {
            let adapter = Self::default();
            let internals = adapter.internals.clone();
            return (adapter, internals);
        }
    }

    impl DebugAdapter for FakeDebugAdapter {
        fn try_receive_message(&self) -> DebugAdapterResult<IncomingMessage> {
            self.internals
                .borrow_mut()
                .receiver_queue
                .pop_front()
                .unwrap_or(Err(TryRecvError::Empty.into()))
        }
        fn send_message(&self, message: OutgoingMessage) -> DebugAdapterResult<()> {
            Ok(self.internals.borrow_mut().sender_queue.push_back(message))
        }
    }

    #[test]
    fn initialization_sequence() {
        let (adapter, adapter_internals) = FakeDebugAdapter::new();
        push_incoming(&*adapter_internals, initialize_request());
        push_incoming(&*adapter_internals, set_exception_breakpoints_request());
        push_incoming(&*adapter_internals, attach_request());
        let mut debugger = Debugger::new(adapter);

        debugger.process_meessages();

        assert_matches!(
            pop_outgoing(&*adapter_internals),
            Some(OutgoingMessage::Initialize(InitializeResponse {
                type_,
                command,
                success: true,
                ..
            })) if type_ == "response" && command == "initialize"
        );
        assert_matches!(
            pop_outgoing(&*adapter_internals),
            Some(OutgoingMessage::Initialized(InitializedEvent {
                type_,
                event,
                ..
            })) if type_ == "event" && event == "initialized"
        );
        assert_matches!(
            pop_outgoing(&*adapter_internals),
            Some(OutgoingMessage::SetExceptionBreakpoints(SetExceptionBreakpointsResponse {
                type_,
                command,
                ..
            })) if type_ == "response" && command == "set_exception_breakpoints"
        );
        assert_matches!(
            pop_outgoing(&*adapter_internals),
            Some(OutgoingMessage::Attach(AttachResponse {
                type_,
                command,
                ..
            })) if type_ == "response" && command == "attach"
        );
        assert_matches!(
            pop_outgoing(&*adapter_internals),
            Some(OutgoingMessage::Stopped(StoppedEvent {
                type_,
                event,
                ..
            })) if type_ == "event" && event == "stopped"
        );
        assert_eq!(pop_outgoing(&*adapter_internals), None);
    }

    #[test]
    fn uses_sequence_numbers() {
        let (adapter, adapter_internals) = FakeDebugAdapter::new();
        push_incoming(&*adapter_internals, initialize_request());
        push_incoming(&*adapter_internals, attach_request());
        let mut debugger = Debugger::new(adapter);

        debugger.process_meessages();

        // TODO: The initialization sequence isn't really good to verify this.
        // Let's use some repeatable messages of the same type.
        assert_matches!(
            pop_outgoing(&*adapter_internals),
            Some(OutgoingMessage::Initialize(InitializeResponse {
                seq: 1,
                request_seq: 5,
                ..
            }))
        );
        assert_matches!(
            pop_outgoing(&*adapter_internals),
            Some(OutgoingMessage::Initialized(InitializedEvent {
                seq: 2,
                ..
            }))
        );
        assert_matches!(
            pop_outgoing(&*adapter_internals),
            Some(OutgoingMessage::Attach(AttachResponse {
                seq: 3,
                request_seq: 8,
                ..
            }))
        );
    }
}
