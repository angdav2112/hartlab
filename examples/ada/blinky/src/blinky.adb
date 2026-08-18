--  PolarFire Icicle blinky (Ada), same contract as examples/rust/blinky.
--  Application hart is U54_1 (mhartid = 1). Other harts should WFI in startup.
--
--  Build (when a GNAT RISC-V zfp runtime is available):
--    alr build
--  Otherwise ship a CI-prebuilt ELF next to this file.

with System.Storage_Elements; use System.Storage_Elements;

procedure Blinky is
   SYSREG            : constant Integer_Address := 16#2000_2000#;
   SUBBLK_CLOCK_CR   : constant Integer_Address := SYSREG + 16#84#;
   SOFT_RESET_CR     : constant Integer_Address := SYSREG + 16#88#;
   GPIO2_CLOCK_BIT   : constant Unsigned_32 := Shift_Left (1, 22);

   GPIO2             : constant Integer_Address := 16#2012_2000#;
   GPIO_OUTP         : constant Integer_Address := GPIO2 + 16#88#;
   GPIO_EN_OUT       : constant Unsigned_32 := 1;
   GPIO_EN_OUT_BUF   : constant Unsigned_32 := 4;
   LED1_PIN          : constant Natural := 16;
   PERIOD_TICKS      : constant Unsigned_64 := 500_000;

   CLINT_MTIME       : constant Integer_Address := 16#0200_BFF8#;

   type U32_Ptr is access all Unsigned_32;
   type U64_Ptr is access all Unsigned_64;
   for U32_Ptr'Storage_Size use 0;
   for U64_Ptr'Storage_Size use 0;

   function To_U32 (Addr : Integer_Address) return U32_Ptr is
   begin
      return U32_Ptr (To_Address (Addr));
   end To_U32;

   function To_U64 (Addr : Integer_Address) return U64_Ptr is
   begin
      return U64_Ptr (To_Address (Addr));
   end To_U64;

   procedure Mmio_Write (Addr : Integer_Address; Value : Unsigned_32) is
   begin
      To_U32 (Addr).all := Value;
   end Mmio_Write;

   function Mmio_Read (Addr : Integer_Address) return Unsigned_32 is
   begin
      return To_U32 (Addr).all;
   end Mmio_Read;

   function Mtime return Unsigned_64 is
   begin
      return To_U64 (CLINT_MTIME).all;
   end Mtime;

   procedure Delay_Ticks (Ticks : Unsigned_64) is
      Start : constant Unsigned_64 := Mtime;
   begin
      while Mtime - Start < Ticks loop
         null;
      end loop;
   end Delay_Ticks;

   On : Boolean := False;
begin
   Mmio_Write (SUBBLK_CLOCK_CR, Mmio_Read (SUBBLK_CLOCK_CR) or GPIO2_CLOCK_BIT);
   Mmio_Write (SOFT_RESET_CR, Mmio_Read (SOFT_RESET_CR) and not GPIO2_CLOCK_BIT);
   Mmio_Write (GPIO2 + 4 * Integer_Address (LED1_PIN), GPIO_EN_OUT or GPIO_EN_OUT_BUF);

   loop
      On := not On;
      declare
         Outp : Unsigned_32 := Mmio_Read (GPIO_OUTP);
      begin
         if On then
            Outp := Outp or Shift_Left (1, LED1_PIN);
         else
            Outp := Outp and not Shift_Left (1, LED1_PIN);
         end if;
         Mmio_Write (GPIO_OUTP, Outp);
      end;
      Delay_Ticks (PERIOD_TICKS);
   end loop;
end Blinky;
