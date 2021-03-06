#!/usr/bin/env bash

n_points=20
n_servers=5
grid_size=3
epochs=10

grid_file="grid/grid.txt"
keys_dir="security/keys"

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
./target/$dir/security --clients $n_points --servers $n_servers --keys $keys_dir

echo "Starting Servers"
echo
rm server/storage/* 2> /dev/null
for ((idx=0;idx<n_servers;idx++))
do
    gnome-terminal -- ./target/$dir/server --id $idx --size $grid_size --keys $keys_dir --fline $f_line --n_servers $n_servers
done

echo "Starting Clients"
echo
for ((idx=0;idx<n_points-1;idx++))
do
    gnome-terminal -- ./target/$dir/client --n_servers $n_servers --id $idx --grid $grid_file --keys $keys_dir
done

echo "Starting ha_client"
echo
./target/$dir/ha_client --n_servers $n_servers --keys $keys_dir