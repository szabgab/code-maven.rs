set -e


function check_project
{
    if ! test -d "$project"; then
        project=../$project
    fi

    echo "---------------------------"
    echo "project: $project"
    echo "folder:  $folder"
    rm -rf _site/*
    cargo run --bin code-maven-web -- --root $project/$folder  --outdir _site/

    echo "---------------------------"
    for page in _site/*.html
    do
        page=$(basename $page)
        echo $page in $project in $project
        page=$(sed "s/html/png/" <<< "$page")
        echo _site/img/$page
        test -f _site/img/$page
    done
}

folder=
for project in rust.org.il izrael.szabgab.com israel.szabgab.com rust.code-maven.com site
#for project in rust.org.il site
do
    check_project
done

# projects where the site is in the "site" subfolder.
folder=site
for project in site-checker.rs
do
    check_project
done


echo "-------------------------------"
echo Finished successfully
