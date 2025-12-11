# SecureArc User Guide

SecureArc is a secure, persistent, and self-destructing archival tool designed for sensitive data storage.

## Installation

### Windows
1. Download `SecureArc_Installer.exe` from the Releases page.
2. Run the installer and follow the on-screen instructions.
3. SecureArc will be added to your Start Menu and Desktop.

### Linux / macOS
(Coming soon)

## Quick Start

### Create an Archive (GUI)
1. Open **SecureArc** from the Start Menu.
2. Go to the **Create Archive** tab.
3. Click **+ Add Files** to select the files you want to secure.
4. Choose an **Output Archive** location (e.g., `C:\MyDocs\secure.sarc`).
5. Enter a strong **Password**.
6. Click **Create Archive**.

### View & Extract (GUI)
1. Double-click a `.sarc` file OR open SecureArc and go to the **View / Info** tab.
2. Enter the password to unlock the archive.
3. To extract files, go to the **Extract Archive** tab, ensure the archive is selected, choose an **Output Directory**, and click **Extract Files**.

## Self-Destruct Mechanism
SecureArc archives are protected by a self-destruct mechanism.
- **Max Attempts**: By default, an archive allows **5 failed password attempts**.
- **Destruction**: On the 5th failure, the encryption headers are **permanently wiped** from the file. The data becomes unrecoverable, even with the correct password.
- **Persistence**: The attempt counter is stored within the archive itself.

## Troubleshooting

- **"Archive Destroyed" Error**: This means the maximum number of password attempts was exceeded. The data is lost.
- **Performance**: Large files may take a moment to compress/encrypt. Please be patient.
