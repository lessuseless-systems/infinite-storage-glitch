#!/usr/bin/env bash

# Array of repos in format "owner/repo"
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

mkdir -p isg-repos-cloned
mkdir -p isg-repos-repomix

cd isg-repos-cloned

for repo in "${repos[@]}"; do
  echo "========================================="
  echo "Processing: $repo"
  echo "========================================="

  # Extract repo name for directory
  repo_name=$(basename "$repo")
  safe_name=$(echo "$repo" | tr '/' '_')

  # Clone the repo if not already cloned
  if [ ! -d "$repo_name" ]; then
    echo "Cloning $repo..."
    git clone "https://github.com/${repo}.git" 2>&1 | head -n 5

    if [ $? -ne 0 ]; then
      echo "❌ Failed to clone $repo"
      continue
    fi
  else
    echo "Already cloned: $repo_name"
  fi

  # Run repomix
  echo "Running repomix on $repo_name..."
  cd ..
  nix run nixpkgs#repomix -- "isg-repos-cloned/$repo_name" -o "isg-repos-repomix/${safe_name}.txt"

  if [ $? -eq 0 ]; then
    echo "✅ Successfully processed $repo"
  else
    echo "❌ Failed to process $repo with repomix"
  fi

  cd isg-repos-cloned
  echo ""
done

cd ..
echo "========================================="
echo "✨ All repositories processed!"
echo "========================================="
echo "Cloned repos in: isg-repos-cloned/"
echo "Repomix outputs in: isg-repos-repomix/"
ls -lh isg-repos-repomix/
