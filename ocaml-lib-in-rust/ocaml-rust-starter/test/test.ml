open Ocaml_lib_in_rust
open Bigarray

let shared_mem_holder = ref None

let send_order ~ba ~start ~price ~qty ~side ~type_id = 
  ba.{start} <- (Int64.of_int price);
  ba.{start + 8} <- (Int64.of_int qty);
  ba.{start + 16} <- (Int64.of_int side);
  ba.{start + 17} <- (Int64.of_int type_id);
  (* tell rust that  we need to execute *)
  ba.{18} <- Int64.of_int 0;
  ()


let run_test () =
  let path = "/dev/shm/hft_test" in
  let size = 1024 in (* 1KB is plenty for a few slots *)
  
  (* 1. Create/Open the shared memory file *)
  let fd = Unix.openfile path [Unix.O_RDWR; Unix.O_CREAT] 0o666 in
  
  (* 2. Map it as a Bigarray of 64-bit ints *)
  let ba = array1_of_genarray (Unix.map_file fd int64 c_layout true [|size|]) in
  shared_mem_holder := Some ba;

  spawn_worker ba; (* from lib.rs*)

  Printf.printf "Rust worker spawned. Starting benchmark...\n%!";

  let iterations = 1_000_000 in
  let start_time = Unix.gettimeofday () in

  for i = 1 to iterations do
    let price = i in
    (* Write to Slot 0 (Command) *)
    send_order ~ba:ba ~start:0 ~price:price ~qty:4 ~side:2 ~type_id:0 ; 
   (* 
    print_endline ("ocaml sees at 0: " ^ Int64.to_string ba.{0});
    print_endline ("ocaml sees at 16: " ^ Int64.to_string ba.{16});
    *)
    while ba.{18} <> Int64.mul (Int64.of_int price) (Int64.of_int 3) do 
      () 
    done;
    (*
    print_endline ("ocaml sees at 16: " ^ Int64.to_string ba.{16});
    *)
  done;

  let end_time = Unix.gettimeofday () in
  let total_time = end_time -. start_time in
  let avg_latency_ns = (total_time /. float_of_int iterations) *. 1_000_000_000.0 in

  Printf.printf "Completed %d round-trips\n" iterations;
  Printf.printf "Average Latency: %.2f ns\n" avg_latency_ns

let () = run_test ()

let useless () = 
  let x = hello_world () in
  let _ = print_endline (string_of_int x) in 
  let orderbook = get_orderbook () in
  (Array.get orderbook.asks 3) |> string_of_float |> print_endline
    
 
