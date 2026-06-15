rsync -avz --exclude="sync.sh" --exclude="build" . node81:~/demikernel/
rsync -avz build config node81:/demikernel/
rsync -avz build config node82:/demikernel/