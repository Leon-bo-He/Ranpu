import { invoke } from './invoke';
import type { CalculationResultView, CustomerCodeMatchView } from './types';

export const calculationApi = {
  calculate: (internalColorCode: string, targetKg: number) =>
    invoke<CalculationResultView>('cmd_calculate', {
      cmd: { internal_color_code: internalColorCode, target_kg: targetKg },
    }),

  searchByCustomerCode: (customerColorCode: string) =>
    invoke<CustomerCodeMatchView[]>('cmd_search_by_customer_code', {
      cmd: { customer_color_code: customerColorCode },
    }),
};
