set -e

function run_tests() {
    cargo test
}

function generate() {
    rm -rf temp/
    cargo build --release
    ./target/release/code-maven web --root $source --outdir temp/
    rm -rf temp/img
    echo "---------------------------------"
    diff -r generated/$param temp
}

#if [ "$*" == "" ]
#then
#    echo "Missing parameters from the command line"
#    exit 1
#fi

if [ "$*" == "" ]
then
    params="test demo"
else
    params=$@
fi

#echo "params: $params"

for param in $params
do
    #echo "param: $param"

    if [ $param == "test" ]
    then
        run_tests
        continue
    fi

    source=test_cases/$param/
    generate
done


