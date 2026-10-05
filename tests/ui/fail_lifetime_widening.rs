// error-pattern: lifetime may not live long enough
mod support;fn widen<'a,'b>(p:support::policy::Plan<'a>)->support::policy::Plan<'b> where 'a:'b {p}fn main(){}
