#!/bin/bash
source "$(dirname "$0")/test-lib.sh"

echo "=== Django dispatcher: flat resolution ==="
setup_temp

cat > pyproject.toml << 'EOF'
[project]
name = "demo"

[tool.pdm.scripts]
dev = "src/manage.py runserver"
test = "pytest"
EOF

mkdir -p src
# Use a bash-shebanged manage.py so the shell test doesn't depend on python
# being installed. Kyle doesn't care what interpreter the shebang points at.
cat > src/manage.py << 'EOF'
#!/bin/bash
echo "manage.py called: $*"
EOF
chmod +x src/manage.py

mkdir -p src/app/management/commands
touch src/app/management/commands/__init__.py
cat > src/app/management/commands/exportxml.py << 'EOF'
EOF
cat > src/app/management/commands/seed.py << 'EOF'
EOF

$KYLE 2>&1 | grep -q "dev" && pass "django: lists regular tasks" || fail "django: lists regular tasks"
out=$($KYLE exportxml 2>&1)
echo "$out" | grep -q "manage.py called: exportxml" && pass "django: flat kyle exportxml resolves" || { echo "$out"; fail "django: flat kyle exportxml resolves"; }
out=$($KYLE seed 2>&1)
echo "$out" | grep -q "manage.py called: seed" && pass "django: flat kyle seed resolves" || fail "django: flat kyle seed resolves"

echo ""
echo "=== Django dispatcher: qualified resolution ==="
out=$($KYLE dev:exportxml 2>&1)
echo "$out" | grep -q "manage.py called: exportxml" && pass "django: qualified dev:exportxml" || { echo "$out"; fail "django: qualified dev:exportxml"; }

echo ""
echo "=== Django dispatcher: arg passthrough ==="
out=$($KYLE exportxml --verbose --flag 2>&1)
echo "$out" | grep -q "manage.py called: exportxml --verbose --flag" && pass "django: arg passthrough" || { echo "$out"; fail "django: arg passthrough"; }

echo ""
echo "=== Shadow rule: explicit local task beats dispatcher sub ==="
cat > pyproject.toml << 'EOF'
[project]
name = "demo"

[tool.pdm.scripts]
dev = "src/manage.py runserver"
seed = "echo regular-seed-task"
EOF
# seed is BOTH a pdm task (echo ...) AND a Django management command (src/app/management/commands/seed.py).
# The explicit pdm task must win. The Django version stays reachable via dev:seed.
out=$($KYLE seed 2>&1)
echo "$out" | grep -q "regular-seed-task" && pass "shadow: local task runs (not dispatcher sub)" || { echo "$out"; fail "shadow: local task runs"; }
out=$($KYLE dev:seed 2>&1)
echo "$out" | grep -q "manage.py called: seed" && pass "shadow: dispatcher still reachable via dev:seed" || { echo "$out"; fail "shadow: dispatcher still reachable"; }

echo ""
echo "=== True ambiguity: two dispatcher tasks with different exec_prefix ==="
# Create a second manage.py under a different path so dedupe can't merge them.
mkdir -p other
cat > other/manage.py << 'EOF'
#!/bin/bash
echo "other manage.py called: $*"
EOF
chmod +x other/manage.py
mkdir -p other/app/management/commands
touch other/app/management/commands/__init__.py
cat > other/app/management/commands/exportxml.py << 'EOF'
EOF
cat > pyproject.toml << 'EOF'
[project]
name = "demo"

[tool.pdm.scripts]
dev = "src/manage.py runserver"
alt = "other/manage.py runserver"
EOF
out=$($KYLE exportxml 2>&1)
echo "$out" | grep -q "ambiguous" && pass "ambiguous: two different dispatchers" || { echo "$out"; fail "ambiguous: two different dispatchers"; }
echo "$out" | grep -q "kyle dev:exportxml" && pass "ambiguous: suggests dev:exportxml" || fail "ambiguous: suggests dev:exportxml"
echo "$out" | grep -q "kyle alt:exportxml" && pass "ambiguous: suggests alt:exportxml" || fail "ambiguous: suggests alt:exportxml"
out=$($KYLE alt:exportxml 2>&1)
echo "$out" | grep -q "other manage.py called: exportxml" && pass "ambiguous: alt:exportxml picks alt" || { echo "$out"; fail "ambiguous: alt:exportxml picks alt"; }

cleanup_temp
