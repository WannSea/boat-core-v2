#!/bin/bash

# BMS_ID_Serial_Number_Answer for BMS ID #1
cansend vcan0 00001024#33.33.33.33.AA.AA.AA.AA


# ================================================
# GLOBAL MESSAGES
# ================================================

# EMS_CONTROL
cansend vcan0 401#11.11.22.22.00.00.00.00

# BMS_GLOBAL_STATUS_3
cansend vcan0 404#00.11.22.33.44.55.66.77

# BMS_GLOBAL_STATUS_4
cansend vcan0 404#00.11.22.33.44.55.66.77

# BMS_GLOBAL_STATUS_5
cansend vcan0 404#00.11.22.33.44.55.66.77

# ================================================



# ================================================
# Individual MESSAGES (examples for ID 1)
# ================================================

# BMS_ID_Internal_Status_1
cansend vcan0 00001020#FF.00.FF.AA.00.00.00.00

# BMS_ID_V_01_04
cansend vcan0 00001002#11.11.22.22.33.33.44.44

# BMS_ID_V_05_08
cansend vcan0 00001003#11.11.22.22.33.33.44.44

# BMS_ID_V_09_12
cansend vcan0 00001004#11.11.22.22.33.33.44.44

# BMS_ID_V_13_16
cansend vcan0 00001005#11.11.22.22.33.33.44.44

# BMS_ID_V_21_24
cansend vcan0 00001007#AA.AA.BB.BB.33.33.55.55

# BMS_ID_T_01_06
cansend vcan0 00001009#AA.BB.CC.00.00.00.00.00

# ================================================
