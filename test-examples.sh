for dir in examples/*/; do
    echo "Running tests in $dir"
    (cd "$dir" && make test)
done

if git status --porcelain | grep -q '^[AM]'; then
    echo "There are new or modified files in the examples directories."
    exit 1
else
    echo "All tests passed and no new or modified files found."
fi