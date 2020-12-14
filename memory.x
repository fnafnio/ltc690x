MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  /* These values correspond to the NRF52840 with Softdevices S140 7.0.1 */
  /* FLASH : ORIGIN = 0x00030000, LENGTH = 832K */
  FLASH : ORIGIN = 0x00027000, LENGTH = 832k /*-> 7.0.1 but doesnt work... */
  /* RAM : ORIGIN = 0x20020000, LENGTH = 128K */
  RAM : ORIGIN = 0x2000F000, LENGTH = 128K
}