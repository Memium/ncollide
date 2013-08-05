#[doc(hidden)]; // XXX remove that when development is complete

use std::managed;
use nalgebra::traits::norm::Norm;
use nalgebra::traits::translation::Translation;
use bounding_volume::bounding_volume::BoundingVolume;

enum UpdateState {
    NeedsShrink,
    UpToDate
}

pub enum DBVTNode<V, B, BV> {
    Internal(@mut DBVTInternal<V, B, BV>),
    Leaf(@mut DBVTLeaf<V, B, BV>)
}

pub struct DBVTInternal<V, B, BV> {
    bounding_volume: BV,
    center:          V,
    left:            DBVTNode<V, B, BV>,
    right:           DBVTNode<V, B, BV>,
    parent:          Option<@mut DBVTInternal<V, B, BV>>,
    state:           UpdateState
}

impl<BV: Translation<V>, B, V> DBVTInternal<V, B, BV> {
    pub fn new(bounding_volume: BV,
               parent:          Option<@mut DBVTInternal<V, B, BV>>,
               left:            DBVTNode<V, B, BV>,
               right:           DBVTNode<V, B, BV>)
               -> DBVTInternal<V, B, BV> {
        DBVTInternal {
            center:          bounding_volume.translation(),
            bounding_volume: bounding_volume,
            left:            left,
            right:           right,
            parent:          parent,
            state:           UpToDate
        }
    }
}

pub struct DBVTLeaf<V, B, BV> {
    bounding_volume: BV,
    center:          V,
    object:          B,
    parent:          Option<@mut DBVTInternal<V, B, BV>>,
}


impl<V, B, BV> DBVTNode<V, B, BV> {
    pub fn depth(&self) -> uint {
        match *self {
            Internal(i) => (1 + i.right.depth()).max(&(1 + i.left.depth())),
            Leaf(_)     => 1    
        }
    }

    pub fn num_leaves(&self) -> uint {
        match *self {
            Internal(i) => i.right.num_leaves() + i.left.num_leaves(),
            Leaf(_)     => 1
        }
    }
}

impl<V, B, BV> DBVTInternal<V, B, BV> {
    fn other(@mut self, a: @mut DBVTLeaf<V, B, BV>) -> DBVTNode<V, B, BV> {
        match self.left {
            Leaf(l) => if managed::mut_ptr_eq(a, l) { return self.right },
            _ => { }
        }

        match self.right {
            Leaf(r) => if managed::mut_ptr_eq(a, r) { return self.left },
            _ => { }
        }

        fail!("This node is not the child of its father (something wrong on mother side?).")
    }

    fn other_internal(@mut self, a: @mut DBVTInternal<V, B, BV>) -> DBVTNode<V, B, BV> {
        match self.left {
            Internal(i) => if managed::mut_ptr_eq(a, i) { return self.right },
            _ => { }
        }

        match self.right {
            Internal(i) => if managed::mut_ptr_eq(a, i) { return self.left },
            _ => { }
        }

        fail!("This node is not the child of its father (something wrong on mother side?).")
    }

    fn is_right_internal_node(&self, r: @mut DBVTInternal<V, B, BV>) -> bool
    {
        match self.right {
            Internal(i) => managed::mut_ptr_eq(i, r),
            _ => false
        }
    }
}

impl<V, B, BV: Translation<V>> DBVTLeaf<V, B, BV> {
    pub fn new(bounding_volume: BV, object: B) -> DBVTLeaf<V, B, BV> {
        DBVTLeaf {
            center:          bounding_volume.translation(),
            bounding_volume: bounding_volume,
            object:          object,
            parent:          None
        }
    }

    pub fn unlink(@mut self, curr_root: DBVTNode<V, B, BV>) -> Option<DBVTNode<V, B, BV>> {
        match self.parent {
            Some(p) => {
                self.parent = None;

                let other = p.other(self);

                match p.parent {
                    Some(pp) => {
                        // we are far away from the root
                        if pp.is_right_internal_node(p) {
                            pp.right = other
                        }
                        else {
                            pp.left = other
                        }

                        match other {
                            Internal(i) => i.parent = Some(pp),
                            Leaf(l)     => l.parent = Some(pp),
                        }

                        pp.state = NeedsShrink;

                        Some(curr_root)
                    },
                    None => {
                        // the root changes to the other child
                        match other {
                            Internal(i) => i.parent = None,
                            Leaf(l)     => l.parent = None,
                        }

                        Some(other)
                    }
                }
            },
            None => {
                self.parent = None;

                // the tree becomes empty
                None
            }
        }
    }
}

impl<BV: BoundingVolume, B, V: Sub<V, V> + Norm<N>, N> DBVTNode<V, B, BV> {
    fn sqdist_to(&self, to: &V) -> N {
        match *self {
            Internal(i) => (i.center - *to).sqnorm(),
            Leaf(l)     => (l.center - *to).sqnorm()
        }
    }

    fn enclosing_volume(&self, other: &DBVTNode<V, B, BV>) -> BV {
        match (*self, *other) {
            (Internal(a), Internal(b)) => a.bounding_volume.merged(&b.bounding_volume),
            (Leaf(a)    , Internal(b)) => a.bounding_volume.merged(&b.bounding_volume),
            (Internal(a), Leaf(b))     => a.bounding_volume.merged(&b.bounding_volume),
            (Leaf(a)    , Leaf(b))     => a.bounding_volume.merged(&b.bounding_volume)
        }
    }
}

impl<BV: Translation<V> + BoundingVolume,
     B,
     V: Sub<V, V> + Norm<N>,
     N: Ord>
DBVTInternal<V, B, BV> {
    fn is_closest_to_left(&self, pt: &V) -> bool {
        self.right.sqdist_to(pt) > self.left.sqdist_to(pt)
    }

    fn is_closest_to_right(&self, pt: &V) -> bool {
        !self.is_closest_to_left(pt)
    }

    fn partial_optimise(&mut self) {
        match self.state {
            NeedsShrink => {
                self.bounding_volume = self.right.enclosing_volume(&self.left);
                self.state = UpToDate;
            }
            _ => { }
        }
    }

    fn check_invariant(&self) {
        match (self.right, self.left) {
            (Leaf(a), Leaf(b)) =>
                assert!(self.bounding_volume.contains(&a.bounding_volume) &&
                        self.bounding_volume.contains(&b.bounding_volume)),
            (Internal(a), Leaf(b)) =>
                assert!(self.bounding_volume.contains(&a.bounding_volume) &&
                        self.bounding_volume.contains(&b.bounding_volume)),
            (Leaf(a), Internal(b)) =>
                assert!(self.bounding_volume.contains(&a.bounding_volume) &&
                        self.bounding_volume.contains(&b.bounding_volume)),
            (Internal(a), Internal(b)) =>
                assert!(self.bounding_volume.contains(&a.bounding_volume) &&
                        self.bounding_volume.contains(&b.bounding_volume)),
        }
    }

}

impl<BV: 'static + BoundingVolume + Translation<V>,
     B:  'static,
     V:  'static + Sub<V, V> + Norm<N>,
     N:  Ord>
DBVTNode<V, B, BV> {
    pub fn insert(&mut self, to_insert: @mut DBVTLeaf<V, B, BV>) -> @mut DBVTInternal<V, B, BV> {
        match *self {
            Internal(i) => {
                i.bounding_volume.merge(&to_insert.bounding_volume);

                // iteratively go to the leaves
                let mut curr;
                let mut left;

                if i.is_closest_to_left(&to_insert.center) {
                    curr = i.left;
                    left = true;
                }
                else {
                    curr = i.right;
                    left = false;
                }

                let mut parent = i;

                loop {
                    match curr
                    {
                        Internal(ci) => {
                            // FIXME: we could avoid the systematic merge
                            ci.bounding_volume.merge(&to_insert.bounding_volume);

                            if ci.is_closest_to_left(&to_insert.center) { // FIXME
                                curr = ci.left;
                                left = true;
                            }
                            else {
                                curr = ci.right;
                                left = false;
                            }

                            parent = ci;
                        },
                        Leaf(l) => {
                            let internal = @mut DBVTInternal::new(
                                l.bounding_volume.merged(&to_insert.bounding_volume),
                                Some(parent),
                                Leaf(l),
                                Leaf(to_insert)
                            );

                            l.parent         = Some(internal);
                            to_insert.parent = Some(internal);

                            if left {
                                parent.left = Internal(internal)
                            }
                            else {
                                parent.right = Internal(internal)
                            }

                            return i
                        }
                    }
                }
            },
            Leaf(i) => {
                // create the root
                let root = @mut DBVTInternal::new(
                    i.bounding_volume.merged(&to_insert.bounding_volume),
                    None,
                    Leaf(i),
                    Leaf(to_insert)
                );

                i.parent         = Some(root);
                to_insert.parent = Some(root);

                root
            }
        }
    }

    pub fn interferences_with_leaf(&self,
                                   to_test: @mut DBVTLeaf<V, B, BV>,
                                   out:     &mut ~[(@mut DBVTLeaf<V, B, BV>)])
    {
        match *self
        {
            Internal(i) => {
                i.partial_optimise();

                if i.bounding_volume.intersects(&to_test.bounding_volume)
                {
                    i.left.interferences_with_leaf(to_test, out);
                    i.right.interferences_with_leaf(to_test, out);
                }
            },
            Leaf(l) =>
                if !managed::mut_ptr_eq(l, to_test) &&
                    l.bounding_volume.intersects(&to_test.bounding_volume) {
                    out.push(l)
                }
        }
    }

    pub fn interferences_with_tree(&self,
                                   to_test: &DBVTNode<V, B, BV>,
                                   out:     &mut ~[(@mut DBVTLeaf<V, B, BV>)])
    {
        match (*self, *to_test)
        {
            (Leaf(_), Leaf(lb))          => self.interferences_with_leaf(lb, out),
            (Leaf(la), Internal(_))      => to_test.interferences_with_leaf(la, out),
            (Internal(_), Leaf(lb))      => self.interferences_with_leaf(lb, out),
            (Internal(la), Internal(lb)) => {
                la.partial_optimise();
                lb.partial_optimise();

                if !managed::mut_ptr_eq(la, lb) &&
                    la.bounding_volume.intersects(&lb.bounding_volume)
                {
                    la.right.interferences_with_tree(&lb.right, out);
                    la.left.interferences_with_tree(&lb.left, out);
                    la.right.interferences_with_tree(&lb.left, out);
                    la.left.interferences_with_tree(&lb.right, out);
                }
            },
        }
    }

    pub fn check_invariant(&self) {
        match *self {
            Internal(i) => {
                match i.parent {
                    Some(p) => {
                        p.other_internal(i);
                        assert!(p.bounding_volume.contains(&i.bounding_volume))
                    },
                    None    => { }
                }

                i.check_invariant();
                i.right.check_invariant();
                i.left.check_invariant();
            },
            Leaf(l) => {
                match l.parent {
                    Some(p) => {
                        p.other(l);
                        assert!(p.bounding_volume.contains(&l.bounding_volume))
                    },
                    None    => { }
                }
            }
        }
    }
}