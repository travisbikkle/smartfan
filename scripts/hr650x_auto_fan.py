import subprocess
import re
import sys

import yaml
import time
import datetime
import os

cpu2_fan_speed_set = False
in_band = False


def get_timestamp():
    return datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S")


def get_temperature_and_cpu_num(ipmi_tool_cmd):
    cpu_num = 2
    cmd = f"{ipmi_tool_cmd} sensor | grep CPU | grep Temp"
    process = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, shell=True)
    output, error = process.communicate()

    if process.returncode != 0:
        print(f"Error executing command: {cmd}. Error: {error}")
        return None, 0

    output = output.decode('utf-8')
    lines = output.split('\n')
    temperatures = []

    for line in lines:
        if len(line) == 0:
            continue
        try:
            if line.split('|')[1].strip() == 'na':
                temperatures.append(float(0))
                print('The system is off, tempature is na')
                continue
        except IndexError as err:
            print(f"Error: {err}")
            print(line)
            continue
        if 'Temp' in line:
            temp = re.findall(r'\d+\.\d+', line)
            if temp:
                temperatures.append(float(temp[0]))
                if 'CPU2_Temp' in line and float(temp[0]) == 0:
                    cpu_num = 1

    if not temperatures:
        print("No temperature data found.")
        return None, 0

    # print(lines)
    # print(temperatures)

    return max(temperatures), cpu_num


def set_fan_speed(speed, ipmi_tool_cmd, cpu_num=2):
    global cpu2_fan_speed_set
    print(f'cpu number is {cpu_num}, cpu 2 fan turned off ? {cpu2_fan_speed_set}')
    if cpu_num == 1:
        cmd = f"{ipmi_tool_cmd} raw 0x2e 0x30 00 01 {speed}"
        cmd += f";{ipmi_tool_cmd} raw 0x2e 0x30 00 02 {speed}"
        cmd += f";{ipmi_tool_cmd} raw 0x2e 0x30 00 03 {speed}"
        if not cpu2_fan_speed_set:
            # turn off cpu2 fans
            cmd += f";{ipmi_tool_cmd} raw 0x2e 0x30 00 04 02"
            cmd += f";{ipmi_tool_cmd} raw 0x2e 0x30 00 05 02"
            cmd += f";{ipmi_tool_cmd} raw 0x2e 0x30 00 06 02"
            cpu2_fan_speed_set = True
    else:
        # all fans
        cmd = f"{ipmi_tool_cmd} raw 0x2e 0x30 00 00 {speed}"
    process = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, shell=True)
    output, error = process.communicate()

    if process.returncode != 0:
        print(f"Error executing command: {cmd}. Error: {error}")
        return False

    return True


def get_fan_speed(temp, fan_speeds):
    for fan_speed in fan_speeds:
        if fan_speed['temp_range'][0] <= temp < fan_speed['temp_range'][1]:
            return fan_speed['speed']
    return 100


def main():
    global in_band
    if len(sys.argv) > 1 and sys.argv[1] == "in-band":
        in_band = True
        print("running with in-band mode")
    else:
        print("running with out-band mode")
    while True:
        print(get_timestamp(), end=' ')
        do()
        time.sleep(10)


def do():
    global in_band

    with open(os.path.join(os.path.dirname(__file__), 'HR650X.yaml'), 'r') as file:
        config = yaml.safe_load(file)
        ipmi_host_info = config['ipmi']
        ipmi_tool_cmd = f"ipmitool -I lanplus -H {ipmi_host_info['host']} -U {ipmi_host_info['username']} -P '{ipmi_host_info['password']}'"
    if in_band:
        ipmi_tool_cmd = "ipmitool"

    temp, cpu_num = get_temperature_and_cpu_num(ipmi_tool_cmd)

    if temp is None or cpu_num == 0:
        print("No temperature data found.")
        return

    speed = get_fan_speed(temp, config['fan_speeds'])

    if set_fan_speed(speed, ipmi_tool_cmd, cpu_num):
        print(f"Set fan speed to {speed}% for CPU temperature {temp}°C")


if __name__ == "__main__":
    main()
