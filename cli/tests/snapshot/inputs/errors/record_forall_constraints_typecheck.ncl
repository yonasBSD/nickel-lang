# capture = 'stderr'
# command = ['eval']
let f | forall r. { ; r } -> { x: Number; r } = fun r => %record/insert% "x" r 1 in (f { x = 0 } : _)
