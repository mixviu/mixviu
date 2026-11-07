# Bulk rows

`rows_project(rows, fn(row) = expression)` compiles one bounded expression
plan and evaluates it over an ordered list, without per-row evaluator frames
or a JSON round trip. It returns a new list. Input values keep Mix value
semantics; a refusal returns no partial result.

```mix
$labels = rows_project($messages, fn($m) = {
  key: $m.id,
  cells: [$m.subject ?? "", $m.from[0]?.email ?? ""]
})
```

Supported expressions: scalar literals, the row parameter, lexical captures,
field/index access, maps/lists, binary operators except pipe, and ternary
conditionals. `and`, `or` and `??` short-circuit. Captured containers retain
copy-on-write semantics. Global or module variable lookup is deliberately
not performed: bind a constant to a local before creating the lambda.

Supported pure calls are `fmt`, `date_format`, `lower`, `upper`, `basename`,
`dirname`, `split`, `len`, `contains`, `starts_with`, `ends_with`, `re_replace`,
`slice`, `to_string`, `floor`, `abs`, `min` and `max`. Their normal builtin
contracts and capability policy still apply. There are no user-function,
extension, IO, shell or Bus calls inside a plan. Unsupported shapes raise
`ROW_EXPRESSION` before any row is processed, including for an empty list.
Use ordinary `map` for unrestricted Mix behaviour.

A call accepts at most 65,536 rows and an expression lambda with a row
parameter and optional second, zero-based index parameter. For example,
`rows_project($names, fn($name, $i) = {name:$name, size:$stats[$i].size})`
joins aligned batches without an interpreted loop. The caller owns the join
and field-selection policy.
Plans are bounded to 512 syntax nodes and 64 levels. Interruption and the
evaluator deadline are checked between rows, and normal value-size limits
apply during projection. This bounds interpreter work, not the elapsed time
of an individual pure formatter or regex operation.

`stat_many(paths[, {follow_symlinks: bool}])` reads up to 65,536 UTF-8
filesystem paths in input order. Each result is the ordinary `stat` map, or
nil for an IO failure, matching a caller that catches each `stat` refusal.
Argument types, NULs and unknown options refuse the entire call before IO.
The list is not a filesystem snapshot. No traversal, selection, sorting,
hidden-file or presentation policy is applied. Byte-preserving path tokens
remain the owning host's explicit path codec; they are not decoded by this
UTF-8 path primitive.

These are shared engine primitives. Applications choose fields, labels,
formatting, file-type icons and error presentation in Mix.

The Dopus Mix pane uses a separate application-owned traversal work budget:
65,536 enumerated entry visits (matching the embedded evaluator's list
ceiling), maximum expansion depth 64 with root depth zero. Hidden entries
and child-count directory reads consume this total; visiting an expanded
directory again consumes work again. The existing native `viu_files("names")`
primitive receives the remaining limit and stops before allocating past it.
`SCAN_BUDGET` propagates before committing a candidate pane. Smaller explicit
budgets can be supplied by callers/tests; invalid budgets also refuse before
refresh mutates anything. This is a work limit, not silent truncation.
