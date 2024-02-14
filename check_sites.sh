set -e


function check_project
{
    echo "---------------------------"
    echo "site:    $site"
    rm -rf _site/
    cargo run --bin code-maven -- web --root $site  --outdir _site/

    echo "---------------------------"
    for full_path in _site/*.html
    do
        echo $full_path
        page=$(basename $full_path)
        echo $page

        # don't expect to have a png file for pages that redirect
        set +e
        redirect=$(grep 'http-equiv="refresh"' $full_path)
        set -e
        if [ "$redirect" == "" ]
        then
            page=$(sed "s/html$/png/" <<< "$page")
            echo _site/img/$page
            test -f _site/img/$page
        fi
    done
}

        # https://github.com/moshe742/my_blog \
function check_public_projects
{
    for url in \
        https://github.com/szabgab/rust.code-maven.com \
        https://github.com/szabgab/rust.org.il \
        https://github.com/szabgab/banner-builder.rs \
        https://github.com/szabgab/site-checker.rs
    do
        echo "Processing $url"
        cd $root
        folder=${url##*/}
        echo "folder: $folder"
        if [ -e sites/$folder ]
        then
            echo "exists. git pull"
            cd sites/$folder
            git pull
        else
            echo "git clone"
            cd sites
            git clone $url
            cd $folder
        fi
        cd $root
        site=sites/$folder

        if [ "$folder" == "banner-builder.rs" ] || [ "$folder" == "site-checker.rs" ]
        then
            echo "site"
            site=$site/site
        fi

        check_project
    done
}



function check_private_projects
{
    for project in izrael.szabgab.com israel.szabgab.com
    do
        echo "private project $project"
        if [ -e ../$project ];
        then
            site=../$project
        else
            if [ -e sites/$project ];
            then
                site=sites/$project
            else
                echo skipping private site
                continue
            fi
        fi

        check_project
    done
}


root=$(pwd)
echo "root=$root"
mkdir -p sites

site=site
check_project

check_public_projects
check_private_projects



echo "-------------------------------"
echo Finished successfully
