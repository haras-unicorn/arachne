set windows-shell := ["nu.exe", "-c"]
set shell := ["nu", "-c"]

root := absolute_path('')
gitignore := absolute_path('.gitignore')
prettierignore := absolute_path('.prettierignore')
markdown-link-check-rc := absolute_path('.markdown-link-check.json')
docs := absolute_path('docs')
artifacts := absolute_path('artifacts')

default:
    @just --choose

dev *args:
    cd '{{ root }}'; \
      cargo run --bin arachne -- {{ args }}

format:
    cd '{{ root }}'; just --fmt --unstable

    nixpkgs-fmt '{{ root }}'

    try { markdownlint --ignore-path '{{ gitignore }}' '{{ root }}' }

    prettier --write \
      --ignore-path '{{ gitignore }}' \
      --cache --cache-strategy metadata \
      '{{ root }}'

    cd '{{ root }}'; cargo fmt --all

    cd '{{ root }}'; cargo clippy --fix --allow-dirty --allow-staged

lint:
    ((git rev-parse --abbrev-ref HEAD) == main) \
      or (commitlint --from main)

    prettier --check \
      --ignore-path '{{ gitignore }}' \
      --ignore-path '{{ prettierignore }}' \
      --cache --cache-strategy metadata \
      '{{ root }}'

    cspell lint '{{ root }}' \
      --no-progress

    markdownlint --ignore-path '{{ gitignore }}' '{{ root }}'
    markdown-link-check \
      --config '{{ markdown-link-check-rc }}' \
      --quiet ...(fd '.*\.md' | lines)

    cd '{{ root }}'; cargo clippy -- -D warnings

test:
    cd '{{ root }}'; cargo test

wiki:
    mdbook serve '{{ docs }}/wiki'

docs:
    rm -rf '{{ artifacts }}'

    mdbook build '{{ docs }}'
    mv '{{ docs }}/book' '{{ artifacts }}'

    cd '{{ root }}'; cargo doc --no-deps
    mv '{{ root }}/target/doc' '{{ artifacts }}/code'
