#!/usr/bin/env bash

n_points=500
grid_size=5
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
rm single_server/storage.txt
gnome-terminal -- ./target/$dir/single_server --server $server_addr --size $grid_size --keys $keys_dir --fline $f_line


echo "Starting Clients"
echo
for ((idx=0;idx<n_points-1;idx++))
do
    gnome-terminal -- ./target/$dir/client --server $server_url --id $idx --grid $grid_file --keys $keys_dir
done

echo "Starting ha_client"
echo
./target/$dir/ha_client --keys $keys_dir