#!/bin/bash

repos=(
  "4A49/Infinite-Storage-Glitch"
  "KKarmugil/Infinite_Storage_Glitch"
  "Memorix101/infinite-storage-glitch-csharp"
  "Atiseug/py-ISG"
  "Rohit10701/infinite-storage-glitch"
  "thebitanpaul/Infinite-Storage-Glitch"
  "norangeflame/infinite-cloud-storage"
  "ycs77/infinite-storage-glitch-docker"
  "Sberm/Thats-Not-A-Vid.cc"
  "Santhoshkrk/Infinite-Storage-Glitch"
  "dev2180/infinite-storage-glitch"
  "ycs77/Infinite-Storage-Glitch"
  "PrLu/Infinite-Storage-Glitch"
  "harshmohite04/Infinite-Storage-Glitch"
  "unsigned-long-long-int/infinite-storage-glitch"
  "alexanki23890t/infinite-storage-glitch"
  "User1334/Infinite_Storage_Glitcher"
  "ranjeetmalik/Infinite-Storage-Glitch"
  "OM-bit-hub/Infinite_Storage_Glitch"
  "Nick4421/ISG-2.0"
  "Vidyarani11Patil/Infinite-Storage-Glitch-Project"
  "g-utsav/ISG---Infinite-Storage-Glitch"
  "yopremium21/Infinite-Storage-Glitch-youtube"
  "crosshair-01/Infinite-Storage-Glitch-master"
  "Archanadigraj/Infinite-Storage-Glitch-project"
  "knkr1/better-infinite-storage-glitch"
  "VMoorjani/EncryptedInfiniteStorageGlitch"
  "techkamar/isg_magic"
)

for repo in "${repos[@]}"; do
  echo "Processing $repo..."
  repo_name=$(echo "$repo" | tr '/' '_')
  output_file="isg-repos-repomix/${repo_name}.txt"
  
  nix run nixpkgs#repomix "https://github.com/${repo}" -o "$output_file"
  
  if [ $? -eq 0 ]; then
    echo "✓ Successfully processed $repo"
  else
    echo "✗ Failed to process $repo"
  fi
  echo "---"
done

echo "All repositories processed!"
