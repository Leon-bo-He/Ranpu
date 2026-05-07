import { invoke } from './invoke';
import type { CalculationResultView } from './types';

export const calculationApi = {
  calculate: (internalColorCode: string, targetKg: number) =>
    invoke<CalculationResultView>('cmd_calculate', {
      cmd: { internal_color_code: internalColorCode, target_kg: targetKg },
    }),
};
