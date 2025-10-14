#!/bin/bash

clear
echo "====================================================================="
echo "= .___        _____             __________          .___  .___       ="
echo "= |   | _____/ ____\___________ \______   \__ __  __| _/__| _/__.__. ="
echo "= |   |/    \   __\\_  __ \__  \ |    |  _/  |  \/ __ |/ __ <   |  | ="
echo "= |   |   |  \  |   |  | \// __ \|    |   \  |  / /_/ / /_/ |\___  | ="
echo "= ___|___|  /__|   |__|  (____  /______  /____/\____ \____ |/ ____|  ="
echo "=         \/                  \/       \/           \/    \/\/       ="
echo "======================================================================"
echo "                   Gathering System Information                       "
echo "======================================================================"
MAIN_IP=$(ip route get 1.1.1.1 | awk '{print $7}')
NIC=$(ip -o addr show | awk -v ip="$MAIN_IP" '$4 ~ ip"/" {print $2}')
echo "$NIC"
MAIN_DRIVE=$(lsblk -no PATH,MOUNTPOINT | awk '$2=="/"{print $1}')
if [[ "$MAIN_DRIVE" == *"nvme"* ]]; then
    DRIVE_FRIENDLY_NAME="NVMe SSD"
else
    DRIVE_FRIENDLY_NAME="Main Storage"
fi
clear
echo "====================================================================="
echo "= .___        _____             __________          .___  .___       ="
echo "= |   | _____/ ____\___________ \______   \__ __  __| _/__| _/__.__. ="
echo "= |   |/    \   __\\_  __ \__  \ |    |  _/  |  \/ __ |/ __ <   |  | ="
echo "= |   |   |  \  |   |  | \// __ \|    |   \  |  / /_/ / /_/ |\___  | ="
echo "= ___|___|  /__|   |__|  (____  /______  /____/\____ \____ |/ ____|  ="
echo "=         \/                  \/       \/           \/    \/\/       ="
echo "======================================================================"
echo "                       Discord Information                            "
echo "======================================================================"
echo
echo "Paste your System Embed Discord Webhook" 
read SYS_EMBED_WEBHOOK
echo
echo "Paste your SSH Logs Discord Webhook. Leave blank to disable." 
read SSH_EMBED_WEBHOOK
if [[ -z "$SSH_EMBED_WEBHOOK" ]]; then
    SSH_ENABLED="false"
    echo "SSH Embeds Disabled."
else
    SSH_ENABLED="true"
    echo "SSH Embeds Enabled."
fi
echo
read -p "What is this system's name: " SYS_NAME
echo
clear
echo "====================================================================="
echo "= .___        _____             __________          .___  .___       ="
echo "= |   | _____/ ____\___________ \______   \__ __  __| _/__| _/__.__. ="
echo "= |   |/    \   __\\_  __ \__  \ |    |  _/  |  \/ __ |/ __ <   |  | ="
echo "= |   |   |  \  |   |  | \// __ \|    |   \  |  / /_/ / /_/ |\___  | ="
echo "= ___|___|  /__|   |__|  (____  /______  /____/\____ \____ |/ ____|  ="
echo "=         \/                  \/       \/           \/    \/\/       ="
echo "======================================================================"
echo "                      Downloading InfraBuddy                          "
echo "======================================================================"
mkdir -p /etc/infrabuddy
cd /etc/infrabuddy
wget https://github.com/InfraCharm/InfraBuddy/releases/download/v1.0/InfraBuddy.zip
dpkg -s unzip &>/dev/null || sudo apt install -y unzip
cd /etc/infrabuddy
unzip InfraBuddy.zip
chmod +x infrabuddy
cat > config.toml <<CONFIG
# InfraBuddy Hardware Resource Monitor
# Developed by InfraCharm LLC
# https://infracharm.com

############################################
#                                          #
#            Discord Options               #
#                                          #
############################################

webhook_url = "$SYS_EMBED_WEBHOOK"
embed_title = "$SYS_NAME"
embed_color = "#E51E46"
# Do not go below 10s for updates. This will trigger the Discord rate limit.
update_interval = "15s"
update_previous_message = false
message_id = ""

############################################
#                                          #
#            Resource Options              #
#                                          #
############################################

show_memory = true
memory_in_mb = false
show_cpu = true

############################################
#                                          #
#            Network Options               #
#                                          #
############################################

show_network_usage = true
# Network interfaces to include
# Use '[]' (without the quotes) to display all interfaces
network_interfaces = ["$NIC"]

############################################
#                                          #
#            Tagging Options               #
#                                          #
############################################

user_tags_enabled = false
user_tags = ["USER_OR_GROUP_ID"]

############################################
#                                          #
#            Footer Options                #
#                                          #
############################################

optional_message_enabled = true
optional_message = "Thank you for using InfraBuddy!"

############################################
#                                          #
#             Disk Options                 #
#                                          #
############################################

show_disk_usage = true

# Drives to monitor for disk usage
disk_drives = ["$MAIN_DRIVE"]

# Friendly names for disk drives
[disk_names]
"$MAIN_DRIVE" = "$DRIVE_FRIENDLY_NAME"

[ssh_alerts]
enabled = $SSH_ENABLED
log_path = "/var/log/auth.log"  # Update the path if different
ssh_alert_webhook_url = "$SSH_EMBED_WEHBOOK"

CONFIG

clear
echo "====================================================================="
echo "= .___        _____             __________          .___  .___       ="
echo "= |   | _____/ ____\___________ \______   \__ __  __| _/__| _/__.__. ="
echo "= |   |/    \   __\\_  __ \__  \ |    |  _/  |  \/ __ |/ __ <   |  | ="
echo "= |   |   |  \  |   |  | \// __ \|    |   \  |  / /_/ / /_/ |\___  | ="
echo "= ___|___|  /__|   |__|  (____  /______  /____/\____ \____ |/ ____|  ="
echo "=         \/                  \/       \/           \/    \/\/       ="
echo "======================================================================"
echo "                      Creating Systemd Service                        "
echo "======================================================================"
echo
touch /etc/systemd/system/infrabuddy.service
cat > /etc/systemd/system/infrabuddy.service << SYSTEMDSERVICE
[Unit]
Description=InfraBuddy Resource Monitor
After=network.target
Wants=network-online.target

[Service]
Restart=no
Type=simple
ExecStart=/etc/infrabuddy/infrabuddy
WorkingDirectory=/etc/infrabuddy

[Install]
WantedBy=multi-user.target

SYSTEMDSERVICE
clear
echo "====================================================================="
echo "= .___        _____             __________          .___  .___       ="
echo "= |   | _____/ ____\___________ \______   \__ __  __| _/__| _/__.__. ="
echo "= |   |/    \   __\\_  __ \__  \ |    |  _/  |  \/ __ |/ __ <   |  | ="
echo "= |   |   |  \  |   |  | \// __ \|    |   \  |  / /_/ / /_/ |\___  | ="
echo "= ___|___|  /__|   |__|  (____  /______  /____/\____ \____ |/ ____|  ="
echo "=         \/                  \/       \/           \/    \/\/       ="
echo "======================================================================"
echo "                   Discord Configuration (Cont.)                      "
echo "======================================================================"
echo
echo "Starting Infrabuddy Service"
systemctl enable --now infrabuddy.service
sleep 3
echo "Stopping InfraBuddy Service"
systemctl stop infrabuddy.service
echo
echo "Please find the System Monitor Embed in Discord."
echo "Right click the message, and copy the message ID."
echo "Hint: Developer tools needs to be enabled to do this."
echo
read -p "Discord Embed Message ID: " DISCORDEMBEDID
echo
echo "Enabling Automatic Embed Updates"
sed -i 's/^update_previous_message *= *.*/update_previous_message = true/' /etc/infrabuddy/config.toml
echo "Setting the Message ID"
sed -i "s/^message_id *= *.*/message_id = \"${DISCORDEMBEDID}\"/" /etc/infrabuddy/config.toml
systemctl restart infrabuddy.service
sleep 1
clear
echo "====================================================================="
echo "= .___        _____             __________          .___  .___       ="
echo "= |   | _____/ ____\___________ \______   \__ __  __| _/__| _/__.__. ="
echo "= |   |/    \   __\\_  __ \__  \ |    |  _/  |  \/ __ |/ __ <   |  | ="
echo "= |   |   |  \  |   |  | \// __ \|    |   \  |  / /_/ / /_/ |\___  | ="
echo "= ___|___|  /__|   |__|  (____  /______  /____/\____ \____ |/ ____|  ="
echo "=         \/                  \/       \/           \/    \/\/       ="
echo "======================================================================"
echo "                       Installation Completed                         "
echo "======================================================================"