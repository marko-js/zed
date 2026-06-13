; Outline entries, in Zed's outline.scm @item/@name convention (other
; tools ignore this file). Every element appears, labeled with its tag
; name and #id/.class shorthands. Statement tags (import/export/static/…)
; are statements rather than document structure, so they are excluded.

((element
   (tag_name) @name
   [(shorthand_id) (shorthand_class)]* @name) @item
 (#not-any-of? @name "import" "export" "class" "static" "server" "client"))
