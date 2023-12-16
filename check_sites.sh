set -e


for project in izrael.szabgab.com israel.szabgab.com rust.code-maven.com site
#for project in site
do
    if ! test -d "$project"; then
        project=../$project
    fi

    echo "---------------------------"
    echo $project
    rm -rf _site/*
    cargo run --bin code-maven-web -- --root $project  --outdir _site/

    echo "---------------------------"
    for page in _site/*.html
    do
        page=$(basename $page)
        echo $page in $project in $project
        page=$(sed "s/html/png/" <<< "$page")
        echo _site/img/$page
        test -f _site/img/$page
    done

done

echo "-------------------------------"
echo Finished successfully
