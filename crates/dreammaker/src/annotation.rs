//! Data structures for the parser to output mappings from input ranges to AST
//! elements at those positions.

use std::rc::Rc;

use crate::docs::DocCollection;
use interval_tree::{range, IntervalTree, RangeInclusive, RangePairIter};
use serde::Serialize;

use super::ast::*;
use super::Location;

pub type Iter<'a> = RangePairIter<'a, Location, Annotation>;

#[derive(Clone, Debug, Serialize)]
pub enum Annotation {
    // contextual information
    TreeBlock(Vec<Ident>),
    TreePath(bool, Vec<Ident>),
    TypePath(TypePath),
    Variable(Vec<Ident>),
    ProcHeader(Vec<Ident>, usize),
    ProcBody(Vec<Ident>, usize),
    LocalVarScope(VarType, Ident),

    // local information about a specific token
    UnscopedCall(Ident),
    UnscopedVar(Ident),
    ScopedCall(Vec<Ident>, Ident),
    ScopedVar(Vec<Ident>, Ident),
    ParentCall,  // ..
    ReturnVal,  // .
    InSequence(usize),  // where in TreePath or TypePath is this ident

    // a macro is called here, which is defined at this location
    MacroDefinition(Ident),
    MacroUse {
        name: String,
        definition_location: Location,
        docs: Option<Rc<DocCollection>>,
    },

    Include(std::path::PathBuf),
    Resource(std::path::PathBuf),

    // error annotations, mostly for autocompletion
    ScopedMissingIdent(Vec<Ident>),  // when a . is followed by a non-ident
    IncompleteTypePath(TypePath, PathOp),
    IncompleteTreePath(bool, Vec<Ident>),

    ProcArguments(Vec<Ident>, String, usize),  // Vec empty for unscoped call
    ProcArgument(usize),  // where in the prog arguments we are
    ReturnOperation(std::ops::Range<Location>),
    ReturnStatement{ returned_value: Vec<Annotation> },
}


impl Annotation {
    fn resolved(self, annotation_tree: &AnnotationTree) -> Annotation {
        match self {
            Self::ReturnOperation(range) => {
                let annotations_checked =
                    annotation_tree
                    .get_range(range)
                    .into_iter().map(|iter| iter.1.to_owned())
                    .collect::<Vec<_>>();
            Self::ReturnStatement{ returned_value: annotations_checked }
        },
            _ => self,
        }
    }
}

#[derive(Debug)]
pub struct AnnotationTree {
    tree: IntervalTree<Location, Annotation>,
    len: usize,
}

impl Default for AnnotationTree {
    fn default() -> Self {
        AnnotationTree {
            tree: IntervalTree::new(),
            len: 0,
        }
    }
}

impl AnnotationTree {
    pub fn insert(&mut self, place: std::ops::Range<Location>, value: Annotation) {
        self.tree.insert(range(place.start, place.end.pred()), value);
        self.len += 1;
    }

    pub fn merge(&mut self, other: AnnotationTree) {
        self.len += other.len;
        self.tree.merge(other.tree);
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn iter(&self) -> Iter {
        self.tree.iter()
    }

    pub fn get_location(&self, loc: Location) -> Iter {
        self.tree.range(range(loc.pred(), loc))
    }

    pub fn get_range(&self, place: std::ops::Range<Location>) -> Iter {
        self.tree.range(range(place.start, place.end.pred()))
    }

    pub fn get_range_raw(&self, place: RangeInclusive<Location>) -> Iter {
        self.tree.range(place)
    }
}
