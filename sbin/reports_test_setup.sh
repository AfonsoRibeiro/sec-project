#!/usr/bin/env bash

#launches all clients except with id 19 (used for testing)

n_points=20
grid_size=3
epochs=10

grid_file="grid/grid.txt"
keys_dir="security/keys"

server_addr="[::1]:50051"
server_url="http://[::1]:50051"

dir="debug"
#dir="release"

echo "Generating grid"
echo
./target/$dir/grid -s $grid_size -p $n_points -e $epochs -f $grid_file

# retrieves f_line from grid
f_line=$(cat grid/grid.txt | grep -o -E 'f_line\":[0-9]+')
IFS=: read -r trash f_line <<< "$f_line"

echo "Generating keys"
echo
./target/$dir/security --clients $n_points --keys $keys_dir

echo "Starting Server"
echo
rm single_server/storage/*
gnome-terminal -- ./target/$dir/single_server --server $server_addr --id 1 --size $grid_size --keys $keys_dir --fline $f_line

echo "Starting Clients"
echo
for ((idx=0;idx<n_points-1;idx++))
do
    gnome-terminal -- ./target/$dir/client --server $server_url --id $idx --grid $grid_file --keys $keys_dir
done