# sudo_approve

Instead of setting a weak sudo password, ask root for approval.

Enables passwordless sudo without being too much of a security risk.

## Installation

Compile binaries and install them to `/usr/local/bin` or some other global folder.

Make sure the binaries have the right permissions and are owned by root:

```
$ ls -l /usr/local/bin
-rwxr-xr-x 1 root root 5334856 Dec  9 22:32 sudo_approve
-rwxr-xr-x 1 root root 4646240 Dec  9 22:34 sudo_wait_for_approval
```

Add this line at the bottom of `/etc/pam.d/sudo`:

```
auth required pam_exec.so /usr/local/bin/sudo_wait_for_approval
```

## Usage

If the installation was successful, running `sudo ls` as a normal user should not ask for a password, and instead it
should panic with this error:

```
/usr/local/bin/sudo_wait_for_approval failed: exit code 1
sudo: PAM authentication error: Unknown error -1
sudo: a password is required
```

This is because the `sudo_approve` script should be running in the background. Start that script as root, and try `sudo
ls` again.

This time you will not see anything in the user terminal, but the `sudo_approve` script will print something like this:

```
Got connection at 2023-12-09T22:03:22.852319794+00:00 from (unnamed)
Approve? [y/N]
```

Press `y` and enter to approve the sudo request. Now the user command will run successfully, and any further sudo
commands will not require authorization (similarly to how you only need to input the password once in a while).
