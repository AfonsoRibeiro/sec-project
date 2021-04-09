#!/usr/bin/env bash

n_points=20
grid_size=3
epochs=10

grid_file="grid/grid.txt"

server_addr="[::1]:50051"
server_url="http://[::1]:50051"

dir="debug"
#dir="release"

echo "Generating grid"
echo
./target/$dir/grid -s $grid_size -p $n_points -e $epochs -f $grid_file

echo "Starting Server"
echo
gnome-terminal -- ./target/$dir/single_server --server $server_addr --size $grid_size


echo "Starting Clients"
echo
for ((idx=0;idx<n_points;idx++))
do
    gnome-terminal -- ./target/$dir/client --server $server_url --id $idx --grid $grid_file < sbin/noop.txt
done

echo "Starting ha_client"
echo
./target/$dir/ha_client