#!/bin/bash
source "$(dirname "$0")/test-lib.sh"

echo "=== Completion feed: reserved commands ==="
setup_temp

cat > Kylefile << 'EOF'
# kyle: toml
name = "test"
[tasks.build]
run = "echo hi"
EOF
feed=$($KYLE --completion-feed 2>/dev/null)
echo "$feed" | grep -q "^init$" && pass "feed: includes init" || fail "feed: includes init"
echo "$feed" | grep -q "^upgrade$" && pass "feed: includes upgrade" || fail "feed: includes upgrade"
echo "$feed" | grep -q "^build$" && pass "feed: includes local task" || fail "feed: includes local task"

echo ""
echo "=== Completion feed: dispatcher subs (bare + qualified) ==="
rm -f Kylefile

cat > pyproject.toml << 'EOF'
[project]
name = "demo"
[tool.pdm.scripts]
dev = "src/manage.py runserver"
test = "pytest"
EOF
mkdir -p src
cat > src/manage.py << 'EOF'
#!/bin/bash
echo "manage.py: $*"
EOF
chmod +x src/manage.py
mkdir -p src/app/management/commands
: > src/app/management/commands/__init__.py
: > src/app/management/commands/exportxml.py
: > src/app/management/commands/seed.py

feed=$($KYLE --completion-feed 2>/dev/null)
echo "$feed" | grep -q "^dev$" && pass "feed: includes dispatcher task" || fail "feed: includes dispatcher task"
echo "$feed" | grep -q "^dev:exportxml$" && pass "feed: includes qualified dev:exportxml" || fail "feed: includes qualified dev:exportxml"
echo "$feed" | grep -q "^exportxml$" && pass "feed: includes bare exportxml" || fail "feed: includes bare exportxml"
echo "$feed" | grep -q "^dev:seed$" && pass "feed: includes qualified dev:seed" || fail "feed: includes qualified dev:seed"

echo ""
echo "=== Completion feed: shadowed dispatcher subs ==="
# Add a pdm task named 'seed' — this shadows the bare dispatcher sub
cat > pyproject.toml << 'EOF'
[project]
name = "demo"
[tool.pdm.scripts]
dev = "src/manage.py runserver"
seed = "echo regular-seed"
EOF
feed=$($KYLE --completion-feed 2>/dev/null)
echo "$feed" | grep -q "^dev:seed$" && pass "feed: qualified dev:seed still present" || fail "feed: qualified dev:seed still present"
# bare 'seed' should appear once (local task, not dispatcher sub)
count=$(echo "$feed" | grep -c "^seed$")
[ "$count" -eq 1 ] && pass "feed: bare seed appears exactly once" || fail "feed: bare seed appears $count times"

echo ""
echo "=== Completion feed: namespace prefixes and tasks ==="
rm -f pyproject.toml src -rf

mkdir -p backend
cat > backend/package.json << 'EOF'
{"name":"be","scripts":{"build":"echo be-build","test":"echo be-test"}}
EOF
feed=$($KYLE --completion-feed 2>/dev/null)
echo "$feed" | grep -q "^backend:$" && pass "feed: includes namespace prefix backend:" || fail "feed: includes namespace prefix"
echo "$feed" | grep -q "^backend:build$" && pass "feed: includes backend:build" || fail "feed: includes backend:build"

echo ""
echo "=== --summary still works (compat) ==="
rm -rf backend
cat > Kylefile << 'EOF'
# kyle: toml
name = "test"
[tasks.build]
run = "echo hi"
EOF
out=$($KYLE --summary 2>/dev/null)
echo "$out" | grep -q "build" && pass "summary: still returns task names" || fail "summary: still returns task names"
! echo "$out" | grep -q "init" && pass "summary: does not include reserved cmds" || fail "summary: does not include reserved cmds"

cleanup_temp
