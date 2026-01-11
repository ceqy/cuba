#!/bin/bash
set -e

# Starting port
PORT=50053

# Function to scaffold
# define_service <name> <proto_dir> <db_name>
scaffold() {
  local NAME=$1
  local PROTO=$2
  local DB=$3
  
  echo ">>> Processing $NAME (Port: $PORT, DB: $DB)..."
  ./scripts/scaffold_service.sh "$NAME" "$PROTO" "$PORT" "$DB"
  PORT=$((PORT+1))
}

# --- Core ---
# iam-service (50051) - Skipped (Exists)
# gl-service (50052) - Skipped (Exists)
# ap-service (50053) - Pilot (Already done, but script is idempotent-ish)

# --- FI ---
# scaffold "ap-service" "fi/ap" "cuba_fi_ap" # Already done manually
scaffold "co-service" "fi/co" "cuba_fi_co"
scaffold "tr-service" "fi/tr" "cuba_fi_tr"

# --- SC ---
scaffold "vs-service" "sc/vs" "cuba_sc_vs"
scaffold "df-service" "sc/df" "cuba_sc_df"
scaffold "wm-service" "sc/wm" "cuba_sc_wm"
scaffold "tp-service" "sc/tp" "cuba_sc_tp"
scaffold "bt-service" "sc/bt" "cuba_sc_bt"
scaffold "im-service" "sc/im" "cuba_sc_im"

# --- PM ---
scaffold "se-service" "pm/se" "cuba_pm_se"
scaffold "sp-service" "pm/sp" "cuba_pm_sp"
scaffold "ct-service" "pm/ct" "cuba_pm_ct"
scaffold "sa-service" "pm/sa" "cuba_pm_sa"
scaffold "po-service" "pm/po" "cuba_pm_po"
scaffold "iv-service" "pm/iv" "cuba_pm_iv"

# --- SD ---
scaffold "pe-service" "sd/pe" "cuba_sd_pe"
scaffold "rr-service" "sd/rr" "cuba_sd_rr"
scaffold "so-service" "sd/so" "cuba_sd_so"
scaffold "an-service" "sd/an" "cuba_sd_an"

# --- MF ---
scaffold "pp-service" "mf/pp" "cuba_mf_pp"
scaffold "kb-service" "mf/kb" "cuba_mf_kb"
scaffold "om-service" "mf/om" "cuba_mf_om"
scaffold "qi-service" "mf/qi" "cuba_mf_qi"
scaffold "sf-service" "mf/sf" "cuba_mf_sf"

# --- AM ---
scaffold "pm-service" "am/pm" "cuba_am_pm"
scaffold "gs-service" "am/gs" "cuba_am_gs"
scaffold "ah-service" "am/ah" "cuba_am_ah"
scaffold "eh-service" "am/eh" "cuba_am_eh"

# --- CS ---
scaffold "fd-service" "cs/fd" "cuba_cs_fd"
scaffold "wc-service" "cs/wc" "cuba_cs_wc"
scaffold "cb-service" "cs/cb" "cuba_cs_cb"

# --- HR ---
scaffold "ex-service" "hr/ex" "cuba_hr_ex"
scaffold "ta-service" "hr/ta" "cuba_hr_ta"

# --- RD ---
scaffold "pl-service" "rd/pl" "cuba_rd_pl"
scaffold "ps-service" "rd/ps" "cuba_rd_ps"

echo "Batch Scaffold Complete!"
