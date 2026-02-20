/**
 * Lazy-loaded JSON Schema validator for Urd world output.
 *
 * AJV and the schema are only fetched on the first call to validateSchema().
 * The compiled validator is cached for subsequent calls.
 */

type ValidateFn = (data: unknown) => boolean;

let validateFn: ValidateFn | null = null;
let getErrors: (() => unknown[] | null | undefined) | null = null;

export interface ValidationResult {
  valid: boolean;
  errors: string[];
}

export async function validateSchema(
  jsonString: string,
): Promise<ValidationResult> {
  const data = JSON.parse(jsonString);

  if (!validateFn) {
    const { default: Ajv } = await import('ajv/dist/2020');
    const resp = await fetch('/schema/urd-world-schema.json');
    const schema = await resp.json();
    const ajv = new Ajv({ allErrors: true });
    const validate = ajv.compile(schema);
    validateFn = validate;
    getErrors = () => validate.errors;
  }

  const valid = validateFn(data);
  if (valid) return { valid: true, errors: [] };

  const errs = getErrors?.() ?? [];
  return {
    valid: false,
    errors: (errs as Array<{ instancePath?: string; message?: string }>).map(
      (e) => `${e.instancePath || '/'}: ${e.message}`,
    ),
  };
}
