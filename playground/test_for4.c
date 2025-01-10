int main(void){
  int b;
  // No condition.
  for(int a = 0; ;a = a+1) {
    b = a;
    if (a == 5) break;
  }

  return b;
}
