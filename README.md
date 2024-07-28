Persisting in svchost.exe with a Service DLL for rust
This is a quick lab that looks into a persistence mechanism that relies on installing a new Windows service, that will be hosted by an svchost.exe process.

Overview
At a high level, this is how the technique works:

Create a service EvilSvc.dll DLL (the DLL that will be loaded into an svchost.exe) with the code we want executed on each system reboot

Create a new service EvilSvc with binPath= svchost.exe

Add the ServiceDll value to EvilSvc service and point it to the service DLL compiled in step 1

Modify HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Svchost to specify under which group your service should be loaded into

Start EvilSvc service

The EvilSvc is started and its service DLL EvilSvc.dll is loaded into an svchost.exe
