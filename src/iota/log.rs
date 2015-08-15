//! Primitive command log, currently used for undo / redo.
//! This is a deliberately unoptimized representation, for simplicity.  It is by no means final.

use std::mem;

/// Represents a modification of data.
#[derive(Copy, Clone)]
pub enum Change {
    ///Character insertion.
    Insert(usize, char),
    ///Character removal.
    Remove(usize, char),
}

impl Change {
    /// Reverses a change, consuming it in the process
    pub fn reverse(self) -> Change {
        match self {
            Change::Insert(usize, char) => Change::Remove(usize, char),
            Change::Remove(usize, char) => Change::Insert(usize, char),
        }
    }
}

/// Log entry
/// Entries may only be played linearly--they don't make sense out of order.
#[derive(Clone)]
pub struct LogEntry {
    /// The initial point position associated with this log entry.
    ///
    /// The OLD point position.
    init_point: usize,
    /// The NEW point position.
    pub end_point: usize,
    /// The changes associated with this log entry, in order of occurence (an undo will replay
    /// their inverses, backwards).
    pub changes: Vec<Change>,
}

impl LogEntry {
    /// Reverse a log entry, consuming it in the process.
    pub fn reverse(mut self) -> LogEntry {
        self.changes.reverse();
        LogEntry {
            init_point: self.end_point,
            end_point: self.init_point,
            changes: self.changes.into_iter().map( |change| change.reverse() ).collect(),
        }
    }
}

/// A set of `Change`s that should be treated atomically.
///
/// This transaction always has an associated entry log.  When the transaction is dropped, the
/// entries are committed.
pub struct Transaction<'a> {
    /// Currently, only one transaction may be open at a time.
    entries: &'a mut Log,
    /// The LogEntry under construction by the transaction.  Every data modification should be
    /// recorded with the open Transaction.
    entry: LogEntry,
}

impl<'a> Transaction<'a> {
    /// Log a change with this transaction.
    ///
    /// The logging should occur after the change has been executed.  This may eventually allow
    /// rollback in case of failure.
    pub fn log(&mut self, change: Change, idx: usize) {
        self.entry.changes.push(change);
        self.entry.end_point = idx;
    }
}

impl<'a> Drop for Transaction<'a> {
    fn drop(&mut self) {
        // Check to see if there were any changes, and if not return early.
        if self.entry.changes.is_empty() { return }
        // Create the new log entry
        let entry = LogEntry {
            changes: mem::replace(&mut self.entry.changes, Vec::new()),
            .. self.entry
        };
        // Commit the transaction.
        self.entries.undo.push(entry);
        // Clear the redo entries now that the transaction has been committed.
        self.entries.redo.clear();
    }
}

/// Log entries structure.  Just two stacks.
pub struct Log {
    /// Undo log entries--LIFO stack.
    undo: Vec<LogEntry>,
    /// Redo log entries--LIFO stack.  Cleared after a new change (other than an undo or redo)
    /// is committed.
    redo: Vec<LogEntry>,
}

impl Log {
    /// Set up log entries.  They are initially empty.
    pub fn new() -> Log {
        Log {
            undo: Vec::new(),
            redo: Vec::new(),
        }
    }

    /// Start a new transaction.
    ///
    /// This returns a RAII guard that can be used to record edits during the transaction.
    pub fn start<'a, 'b>(&'a mut self, idx: usize) -> Transaction<'a> {
        Transaction {
            entries: self,
            entry: LogEntry {
                init_point: idx,
                end_point: idx,
                changes: Vec::new(),
            }
        }
    }

    /// This reverses the most recent change on the undo stack, places the new
    /// change on the redo stack, and then returns it.  It is the caller's
    /// responsibility to actually perform the change.
    pub fn undo(&mut self) -> Option<LogEntry> {
        match self.undo.pop() {
            Some(change) => {
                let last = self.redo.len();
                self.redo.push(change.reverse());
                Some(self.redo[last].clone())
            },
            None => None
        }
    }
    /// This reverses the most recent change on the redo stack, places the new
    /// change on the undo stack, and then returns it.  It is the caller's
    /// responsibility to actually perform the change.
    pub fn redo(&mut self) -> Option<LogEntry> {
        match self.redo.pop() {
            Some(change) => {
                let last = self.undo.len();
                self.undo.push(change.reverse());
                Some(self.undo[last].clone())
            },
            None => None
        }
    }
}
