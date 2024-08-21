# InfraBuddy
InfraBuddy Resource Monitor does exactly that - monitors your system's hardware usage and reports it back to you.

## Hardware Monitoring
![image](https://github.com/user-attachments/assets/88189f41-02a1-4478-9ae6-d056bd094cb0)

## SSH Login Alerts
<img width="340" alt="image" src="https://github.com/user-attachments/assets/db8c228e-0eec-4f0e-bc7f-7887d1292384">


## Installation:
1. Create the Directory: `sudo mkdir /etc/infrabuddy`
2. Move infrabuddy executable and config.toml into directory.
3. Give Execute Permissions:`sudo chmod +x /etc/infrabuddy/infrabuddy`
4. Create the Service File: `sudo nano /etc/systemd/system/infrabuddy.service`
5. Paste into Service File: 
```
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
```
5. Enable the Service: `sudo systemctl enable --now infrabuddy`
6. Start the Service: `sudo systemctl start infrabuddy`
