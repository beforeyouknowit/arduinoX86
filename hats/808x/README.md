## ArduinoX86 808X HAT

This HAT supports the 8088, 8086, V20 and V30 CPUs.

It is designed for 3.3V operation.

If using an Arduino Due, you can power the CPU directly from the Arduino. This is the default mode of operation.
Despite being 5V parts, 3.3V operation appears to be fine for CMOS variants of the 8088/8086, the AMD D8088 and the V20
and V30 CPUs.

If using an Arduino GIGA, cut the 3.3V power pin from the headers as the Giga cannot supply enough power to run the CPU.
You can then choose to directly deliver either 3.3 or 5V of external power into the board's 3.3V power header.

> [!WARNING]  
> Do not feed power into the HAT's power header unless you have cut the 3V header pin on the HAT. Feeding power into
> the Arduino's 3V out pin will damage your Arduino.

Please read all the notes in the next section before ordering/assembling parts. Failure to heed warnings will cause
damage to your Arduino.

# BOM

- A compatible CPU. For best results, use a CMOS CPU such as a Harris 80C88, Oki 80C88, or NEC V20 CPU. Beware of
  counterfeits on eBay and other online vendors.
  A legitimate chip will not look shiny and new with perfect printing on it.

- (Optional) An Intel 8288 or OKI 82C88 Bus Controller. If not using an 8288, set the EMULATE_8288 flag in cpu_server.

- A set of Arduino stacking headers (also usable with DUE)
  https://www.amazon.com/Treedix-Stacking-Headers-Stackable-Compatible/dp/B08G4FGBPQ

- A DIP-40 and (optionally) DIP-20 socket
    - Optional: You can spring for a ZIF socket such
      as [https://www.amazon.com/-/en/gp/product/B00B886OZI](https://www.amazon.com/-/en/gp/product/B00B886OZI)

- (2x) 0805 0.047uf bypass capacitors
  https://www.mouser.com/ProductDetail/80-C0805C473KARAUTO

- (Optional) A 12mm, active buzzer with 7.6mm pin spacing.

    - For DUE: A 3V piezoelectric, low power buzzer <= 6mA
      https://www.mouser.com/ProductDetail/Mallory-Sonalert/PK-11N40PQ?qs=SXHtpsd1MbZ%252B7jeUyAAOVA%3D%3D

    - For MEGA: Any 3-5V buzzer <= 30mA
      WARNING: Only connect an electromagnetic buzzer if using an Arduino MEGA. The DUE has much lower GPIO max current
      supply.

- (2x) 750Ohm resistors (for LEDs)
  https://www.mouser.com/ProductDetail/667-ERA-6AED751V

- (2x) Any 0805 ~2V LED of your choice with 1.8-1.9mA forward current
    - https://www.mouser.com/ProductDetail/604-APTD2012LCGCK (Green)
    - https://www.mouser.com/ProductDetail/604-APT2012LSECKJ4RV (Orange)

- RS232 board for debug output - choose gender based on your desired cabling
    - https://www.amazon.com/Ultra-Compact-RS232-Converter-1Mbps/dp/B074BMLM11 (male)
    - https://www.amazon.com/Ultra-Compact-RS232-Converter-Female/dp/B074BTGLJN (female)
    - WARNING: DO NOT connect 5V to rs232 board on DUE

> [!TIP]
> The listed RS232 boards appear to be no longer available. You may try using any of the common MAX3232-based RS232
> breakout boards, but I've had mixed luck with them due to quality issues. The MAX3232 is also less capable so you may
> need to adjust the serial baud rate to be lower.
