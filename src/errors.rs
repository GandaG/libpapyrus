use std::panic;

use annotate_snippets::display_list::DisplayList;
use annotate_snippets::formatter::DisplayListFormatter;
use annotate_snippets::snippet::{Annotation, AnnotationType, Slice, Snippet, SourceAnnotation};

use crate::ParserSession;

struct FatalMarker;

pub struct ErrorBuilder<'a> {
    sess: &'a ParserSession,
    colors: bool,
    fatal: bool,
    title: Option<String>,
    level: AnnotationType,
    lo: usize,
    hi: usize,
    label: Option<String>,
    label_level: Option<AnnotationType>,
}

impl<'a> ErrorBuilder<'a> {
    pub fn new(sess: &'a ParserSession, colors: bool) -> Self {
        Self {
            sess,
            colors,
            fatal: false,
            title: None,
            level: AnnotationType::Info,
            lo: 0,
            hi: 0,
            label: None,
            label_level: None,
        }
    }

    pub fn fatal(&mut self, title: &str) -> &mut Self {
        self.fatal = true;
        self.error(title)
    }

    pub fn error(&mut self, title: &str) -> &mut Self {
        self.title = Some(title.to_string());
        self.level = AnnotationType::Error;
        self
    }

    pub fn warning(&mut self, title: &str) -> &mut Self {
        self.title = Some(title.to_string());
        self.level = AnnotationType::Warning;
        self
    }

    pub fn help(&mut self, title: &str) -> &mut Self {
        self.title = Some(title.to_string());
        self.level = AnnotationType::Help;
        self
    }

    pub fn span(&mut self, lo: usize, hi: usize) -> &mut Self {
        self.lo = lo;
        self.hi = hi;
        self
    }

    pub fn label_error(&mut self, label: &str) -> &mut Self {
        self.label = Some(label.to_string());
        self.label_level = Some(AnnotationType::Error);
        self
    }

    pub fn label_warning(&mut self, label: &str) -> &mut Self {
        self.label = Some(label.to_string());
        self.label_level = Some(AnnotationType::Warning);
        self
    }

    pub fn label_help(&mut self, label: &str) -> &mut Self {
        self.label = Some(label.to_string());
        self.label_level = Some(AnnotationType::Help);
        self
    }

    pub fn emit(&self) {
        let (lo_line, lo_col) = self.sess.src.lineno_from_offset(self.lo);
        let (hi_line, hi_col) = self.sess.src.lineno_from_offset(self.hi);
        let source_list = self.sess.src.lines_from_linenos(lo_line, hi_line);
        if source_list.is_empty() {
            panic!("Source list cannot be empty - internal bug in error creation.")
        }
        let lo = lo_col;
        let source_len = source_list.len();
        let mut hi = hi_col;
        if source_len > 1 {
            // add remainder length of first line
            hi += source_list.first().unwrap().len() - lo_col
        };
        if source_list.len() > 2 {
            // add all line length between first and last
            hi += source_list[1..source_list.len() - 1].iter().map(|x| x.len()).sum::<usize>()
        };
        let formatter = DisplayListFormatter::new(self.colors, false);
        let title = Annotation { id: None, label: self.title.clone(), annotation_type: self.level };
        let annotatation = SourceAnnotation {
            range: (lo, hi),
            label: self.label.clone().unwrap_or_else(|| "".to_string()),
            annotation_type: self.label_level.unwrap_or(self.level),
        };
        let slices = vec![Slice {
            source: source_list.join(""),
            line_start: 1,
            origin: Some(self.sess.src.filename.to_string_lossy().to_string()),
            fold: true,
            annotations: vec![annotatation],
        }];
        let snippet = Snippet { title: Some(title), footer: vec![], slices };
        eprintln!("{}", formatter.format(&DisplayList::from(snippet)));
        if self.fatal {
            panic::resume_unwind(Box::new(FatalMarker));
        }
    }
}
