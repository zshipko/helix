mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

extern crate extism_pdk;

/// Editor context
#[derive(Default, Clone, Copy, Debug)]
pub struct Editor;

/// A text selection
#[derive(Default, Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct Selection(View, u64);

/// A view
pub use bindings::View;

/// Focus handle, this can be used to focus a view
/// and return focus once the `Focus` handle goes
/// out of scope
pub struct Focus(View, View);

impl Focus {
    /// Focus `view`
    pub fn new(view: View) -> Focus {
        let old = Editor.view();
        Editor.focus(view);
        Focus(old, view)
    }

    /// Re-focus
    pub fn focus(&self) {
        Editor.focus(self.1);
    }
}

impl Drop for Focus {
    fn drop(&mut self) {
        Editor.focus(self.0)
    }
}

impl Selection {
    /// Start index
    pub fn from(self) -> usize {
        let _focus = Focus::new(self.0);
        unsafe { bindings::selection_begin(self.1) as usize }
    }

    /// End index
    pub fn to(self) -> usize {
        let _focus = Focus::new(self.0);
        unsafe { bindings::selection_end(self.1) as usize }
    }

    /// Selection text
    pub fn text(&self) -> Result<String, extism_pdk::Error> {
        let _focus = Focus::new(self.0);
        let from = unsafe { bindings::selection_begin(self.1) as usize };
        let to = unsafe { bindings::selection_end(self.1) as usize };
        let ptr = unsafe { bindings::text(from as u64, to as u64) };
        let ptr = extism_pdk::Memory::find(ptr).unwrap();
        let res = ptr.to();
        ptr.free();
        res
    }
}

impl Editor {
    /// Initialize editor context
    pub fn new() -> Editor {
        Editor::default()
    }

    /// Add a new selection
    pub fn add_selection(self, start: usize, end: usize) -> Selection {
        let n = unsafe { bindings::selection_add(start as u64, end as u64) };
        Selection(self.view(), n)
    }

    /// List selections
    pub fn selections(self) -> impl Iterator<Item = Selection> {
        let len = unsafe { bindings::selection_count() };

        let mut n = 0;
        let view = self.view();
        std::iter::from_fn(move || {
            if n < len {
                let x = Selection(view, n);
                n += 1;
                return Some(x);
            }

            None
        })
    }

    pub fn save<P: AsRef<std::path::Path>>(
        self,
        filename: Option<P>,
    ) -> Result<(), extism_pdk::Error> {
        let ptr = if let Some(filename) = filename {
            extism_pdk::Memory::new(&filename.as_ref().to_str().unwrap_or_default())?
        } else {
            extism_pdk::Memory::null()
        };
        unsafe { bindings::save(ptr.offset()) };
        ptr.free();
        Ok(())
    }

    pub fn set_path<P: AsRef<std::path::Path>>(self, filename: P) -> Result<(), extism_pdk::Error> {
        let ptr = extism_pdk::Memory::new(&filename.as_ref().to_str().unwrap_or_default())?;
        unsafe { bindings::set_path(ptr.offset()) }
        ptr.free();
        Ok(())
    }

    pub fn path(self) -> Result<Option<std::path::PathBuf>, extism_pdk::Error> {
        let ptr = unsafe { bindings::get_path() };
        if let Some(ptr) = extism_pdk::Memory::find(ptr) {
            let res: String = ptr.to()?;
            ptr.free();
            if res.is_empty() {
                return Ok(None);
            }
            Ok(Some(std::path::PathBuf::from(res)))
        } else {
            Ok(None)
        }
    }

    pub fn open<P: AsRef<std::path::Path>>(self, filename: P) -> Result<(), extism_pdk::Error> {
        let ptr = extism_pdk::Memory::new(&filename.as_ref().to_str().unwrap_or_default())?;
        unsafe { bindings::open(ptr.offset()) }
        ptr.free();
        Ok(())
    }

    pub fn close(self) {
        unsafe { bindings::close() }
    }

    pub fn undo(self) {
        unsafe { bindings::undo() }
    }

    pub fn redo(self) {
        unsafe { bindings::redo() }
    }

    pub fn view(self) -> View {
        unsafe { bindings::view_id() }
    }

    pub fn focus(self, view: View) {
        unsafe { bindings::focus(view) }
    }

    pub fn focus_next(self) {
        unsafe { bindings::focus_next() }
    }

    pub fn focus_prev(self) {
        unsafe { bindings::focus_prev() }
    }

    pub fn clear_selection(self) {
        unsafe { bindings::selection_reset() }
    }

    pub fn language_name(self) -> Result<String, extism_pdk::Error> {
        let ptr = unsafe { bindings::language_name() };
        let ptr = extism_pdk::Memory::find(ptr).unwrap();
        let res = ptr.to();
        ptr.free();
        res
    }

    pub fn insert_text(
        self,
        text: impl AsRef<str>,
        insert: Insert,
    ) -> Result<(), extism_pdk::Error> {
        let ptr = extism_pdk::Memory::new(&text.as_ref())?;
        match insert {
            Insert::BeforeSelection => unsafe {
                bindings::selection_insert_text_before(ptr.offset())
            },
            Insert::AfterSelection => unsafe {
                bindings::selection_insert_text_after(ptr.offset())
            },
        }
        ptr.free();
        Ok(())
    }

    pub fn replace_text(self, txt: impl AsRef<str>) -> Result<(), extism_pdk::Error> {
        let ptr = extism_pdk::Memory::new(&txt.as_ref())?;
        unsafe { bindings::selection_replace_text(ptr.offset()) };
        ptr.free();
        Ok(())
    }

    pub fn set_status(self, text: impl AsRef<str>) -> Result<(), extism_pdk::Error> {
        let ptr = extism_pdk::Memory::new(&text.as_ref())?;
        unsafe {
            bindings::set_status(ptr.offset());
        }
        ptr.free();
        Ok(())
    }

    pub fn clear_status(self) {
        unsafe {
            bindings::clear_status();
        }
    }

    pub fn execute(self, line: impl AsRef<str>) -> Result<(), extism_pdk::Error> {
        let ptr = extism_pdk::Memory::new(&line.as_ref())?;
        unsafe {
            bindings::execute(ptr.offset());
        }
        Ok(())
    }

    pub fn len_lines(self) -> usize {
        unsafe { bindings::len_lines() as usize }
    }

    pub fn len_chars(self) -> usize {
        unsafe { bindings::len_chars() as usize }
    }

    pub fn len_bytes(self) -> usize {
        unsafe { bindings::len_bytes() as usize }
    }

    pub fn select_all(self) -> Result<(), extism_pdk::Error> {
        self.execute("select_all")
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Command {
    typed: bool,
    args: Vec<String>,
}

impl Command {
    pub fn new(name: impl Into<String>) -> Self {
        Command {
            args: vec![name.into()],
            typed: false,
        }
    }

    pub fn new_typed(name: impl Into<String>) -> Self {
        Command {
            args: vec![name.into()],
            typed: true,
        }
    }

    pub fn typed(&mut self, t: bool) -> &mut Self {
        self.typed = t;
        self
    }

    pub fn arg(&mut self, arg: impl Into<String>) -> &mut Self {
        self.args.push(arg.into());
        self
    }

    pub fn args(&mut self, args: impl IntoIterator<Item = impl Into<String>>) -> &mut Self {
        for arg in args {
            self.args.push(arg.into());
        }
        self
    }

    pub fn execute(&mut self) -> Result<(), extism_pdk::Error> {
        let mut cmd = self.args[0].as_str().to_string();
        if self.typed {
            cmd.insert(0, ':');
        }

        for arg in &self.args[1..] {
            cmd += " \"";
            cmd += arg.as_str();
            cmd += "\"";
        }

        Editor.execute(&cmd)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Insert {
    BeforeSelection,
    AfterSelection,
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
