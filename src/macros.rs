/**
 * A move closure which clones the specified variables before moving them
 */
macro_rules! move_fn_with_clones{
    ($($n:ident),+; || $body:block) => ({
        $( let $n = $n.clone(); )+
        move || { $body }
    });
    ($($n:ident),+; |$($p:pat),+| $body:block) => ({
        $( let $n = $n.clone(); )+
        move |$($p),+| { $body }
    });
}
