#!/usr/bin/env bash

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

echo "Generating keys"
echo
./target/$dir/security --n_clients $n_points --keys $keys_dir

echo "Starting Server"
echo
gnome-terminal -- ./target/$dir/single_server --server $server_addr --size $grid_size --keys $keys_dir


echo "Starting Clients"
echo
for ((idx=0;idx<n_points;idx++))
do
    gnome-terminal -- ./target/$dir/client --server $server_url --id $idx --grid $grid_file --keys $keys_dir < sbin/noop.txt
done

echo "Starting ha_client"
echo
./target/$dir/ha_client --keys $keys_dir