# Quoin source specimen

volume/ contains the existing scene, executable behaviour and template.
lib/ contains its current module-level helper closure.

The helper default resolves relative to the copied bundle; SCENES_LIB can
override it. Execution still requires the existing scene API, runtime and
audio/service facilities. Source preparation does not provide or start them.

This is the first useful shell specimen for later Viu runtime work. It is not
a port, replacement shell, benchmark pass or production install.
